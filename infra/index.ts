import * as pulumi from "@pulumi/pulumi";
import * as aws from "@pulumi/aws";
import * as awsx from "@pulumi/awsx";

// ── Configuration ─────────────────────────────────────────────────────────────
const cfg    = new pulumi.Config();
const awsCfg = new pulumi.Config("aws");

const appName = "forge-fabrik-academy";
const env     = pulumi.getStack();            // dev | staging | prod
const region  = awsCfg.get("region") ?? "eu-central-1";

const dbPassword = cfg.requireSecret("dbPassword");
const jwtSecret  = cfg.requireSecret("jwtSecret");

const containerPort = 8080;
const isProd        = env === "prod";

// ── VPC ───────────────────────────────────────────────────────────────────────
const vpc = new awsx.ec2.Vpc(`${appName}-vpc`, {
    numberOfAvailabilityZones: 2,
    natGateways: { strategy: isProd ? "OnePerAz" : "Single" },
    tags: { Name: `${appName}-vpc-${env}`, Environment: env },
});

// ── Security groups ───────────────────────────────────────────────────────────
const albSg = new aws.ec2.SecurityGroup(`${appName}-alb`, {
    vpcId: vpc.vpcId,
    ingress: [
        { protocol: "tcp", fromPort: 80,  toPort: 80,  cidrBlocks: ["0.0.0.0/0"], description: "HTTP" },
        { protocol: "tcp", fromPort: 443, toPort: 443, cidrBlocks: ["0.0.0.0/0"], description: "HTTPS" },
    ],
    egress: [{ protocol: "-1", fromPort: 0, toPort: 0, cidrBlocks: ["0.0.0.0/0"], description: "all" }],
    tags: { Name: `${appName}-alb-${env}` },
});

const appSg = new aws.ec2.SecurityGroup(`${appName}-app`, {
    vpcId: vpc.vpcId,
    ingress: [{
        protocol: "tcp", fromPort: containerPort, toPort: containerPort,
        securityGroups: [albSg.id], description: "from ALB",
    }],
    egress: [{ protocol: "-1", fromPort: 0, toPort: 0, cidrBlocks: ["0.0.0.0/0"], description: "all" }],
    tags: { Name: `${appName}-app-${env}` },
});

const dbSg = new aws.ec2.SecurityGroup(`${appName}-db`, {
    vpcId: vpc.vpcId,
    ingress: [{
        protocol: "tcp", fromPort: 5432, toPort: 5432,
        securityGroups: [appSg.id], description: "from app",
    }],
    egress: [{ protocol: "-1", fromPort: 0, toPort: 0, cidrBlocks: ["0.0.0.0/0"], description: "all" }],
    tags: { Name: `${appName}-db-${env}` },
});

const redisSg = new aws.ec2.SecurityGroup(`${appName}-redis`, {
    vpcId: vpc.vpcId,
    ingress: [{
        protocol: "tcp", fromPort: 6379, toPort: 6379,
        securityGroups: [appSg.id], description: "from app",
    }],
    egress: [{ protocol: "-1", fromPort: 0, toPort: 0, cidrBlocks: ["0.0.0.0/0"], description: "all" }],
    tags: { Name: `${appName}-redis-${env}` },
});

// ── RDS PostgreSQL ────────────────────────────────────────────────────────────
const dbSubnetGroup = new aws.rds.SubnetGroup(`${appName}-db-subnets`, {
    subnetIds: vpc.privateSubnetIds,
    tags: { Name: `${appName}-db-subnets-${env}` },
});

const db = new aws.rds.Instance(`${appName}-db`, {
    engine:           "postgres",
    engineVersion:    "16.3",
    instanceClass:    isProd ? "db.t4g.medium" : "db.t4g.micro",
    allocatedStorage: 20,
    storageType:      "gp3",
    dbName:           "forge_fabrik_academy",
    username:         "academy",
    password:         dbPassword,
    dbSubnetGroupName:  dbSubnetGroup.name,
    vpcSecurityGroupIds: [dbSg.id],
    multiAz:             isProd,
    skipFinalSnapshot:   !isProd,
    deletionProtection:  isProd,
    backupRetentionPeriod: isProd ? 7 : 1,
    tags: { Name: `${appName}-db-${env}`, Environment: env },
});

// ── ElastiCache Redis ─────────────────────────────────────────────────────────
const redisSubnetGroup = new aws.elasticache.SubnetGroup(`${appName}-redis-subnets`, {
    subnetIds: vpc.privateSubnetIds,
});

const redisCluster = new aws.elasticache.Cluster(`${appName}-redis`, {
    engine:        "redis",
    engineVersion: "7.1",
    nodeType:      "cache.t4g.micro",
    numCacheNodes: 1,
    subnetGroupName:  redisSubnetGroup.name,
    securityGroupIds: [redisSg.id],
    tags: { Name: `${appName}-redis-${env}`, Environment: env },
});

const redisAddr = redisCluster.cacheNodes.apply(nodes => nodes[0].address);

// ── ECR repository ────────────────────────────────────────────────────────────
const repo = new aws.ecr.Repository(`${appName}`, {
    name: appName,
    imageTagMutability: "MUTABLE",
    imageScanningConfiguration: { scanOnPush: true },
    tags: { Name: `${appName}-${env}` },
});

new aws.ecr.LifecyclePolicy(`${appName}-lifecycle`, {
    repository: repo.name,
    policy: JSON.stringify({
        rules: [{
            rulePriority: 1,
            description:  "Keep last 10 images",
            selection:    { tagStatus: "any", countType: "imageCountMoreThan", countNumber: 10 },
            action:       { type: "expire" },
        }],
    }),
});

// ── Secrets Manager ───────────────────────────────────────────────────────────
const secretStore = new aws.secretsmanager.Secret(`${appName}-secrets`, {
    name: `${appName}/${env}`,
    recoveryWindowInDays: isProd ? 7 : 0,
    tags: { Environment: env },
});

// ── CloudWatch log group ──────────────────────────────────────────────────────
const logGroup = new aws.cloudwatch.LogGroup(`${appName}-logs`, {
    name:            `/ecs/${appName}-${env}`,
    retentionInDays: isProd ? 90 : 7,
    tags: { Environment: env },
});

// ── IAM ───────────────────────────────────────────────────────────────────────
const ecsAssumeRole = JSON.stringify({
    Version:   "2012-10-17",
    Statement: [{
        Effect:    "Allow",
        Principal: { Service: "ecs-tasks.amazonaws.com" },
        Action:    "sts:AssumeRole",
    }],
});

const execRole = new aws.iam.Role(`${appName}-exec`, {
    assumeRolePolicy: ecsAssumeRole,
    managedPolicyArns: [
        "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy",
    ],
});

// Allow task executor to read secrets.
new aws.iam.RolePolicy(`${appName}-exec-secrets`, {
    role: execRole.id,
    policy: JSON.stringify({
        Version:   "2012-10-17",
        Statement: [{
            Effect:   "Allow",
            Action:   ["secretsmanager:GetSecretValue"],
            Resource: "*",
        }],
    }),
});

const taskRole = new aws.iam.Role(`${appName}-task`, {
    assumeRolePolicy: ecsAssumeRole,
});

// ── ECS cluster ───────────────────────────────────────────────────────────────
const cluster = new aws.ecs.Cluster(`${appName}-cluster`, {
    name: `${appName}-${env}`,
    settings: [{ name: "containerInsights", value: "enabled" }],
    tags: { Name: `${appName}-cluster-${env}`, Environment: env },
});

// ── ALB ───────────────────────────────────────────────────────────────────────
const alb = new aws.lb.LoadBalancer(`${appName}-alb`, {
    loadBalancerType: "application",
    internal:         false,
    securityGroups:   [albSg.id],
    subnets:          vpc.publicSubnetIds,
    tags: { Name: `${appName}-alb-${env}`, Environment: env },
});

const tg = new aws.lb.TargetGroup(`${appName}-tg`, {
    port:        containerPort,
    protocol:    "HTTP",
    targetType:  "ip",
    vpcId:       vpc.vpcId,
    healthCheck: {
        path:               "/health",
        healthyThreshold:   2,
        unhealthyThreshold: 3,
        interval:           30,
        timeout:            5,
    },
    tags: { Name: `${appName}-tg-${env}` },
});

const httpListener = new aws.lb.Listener(`${appName}-http`, {
    loadBalancerArn: alb.arn,
    port:            80,
    protocol:        "HTTP",
    defaultActions:  [{ type: "forward", targetGroupArn: tg.arn }],
});

// ── ECS task definition ───────────────────────────────────────────────────────
const containerDefs = pulumi.all([
    repo.repositoryUrl, db.endpoint, dbPassword, redisAddr, jwtSecret, logGroup.name,
]).apply(([imageUrl, dbEp, dbPwd, redisEp, jwt, lgName]) =>
    JSON.stringify([{
        name:         "server",
        image:        `${imageUrl}:latest`,
        essential:    true,
        portMappings: [{ containerPort, protocol: "tcp" }],
        environment:  [
            { name: "DATABASE_URL", value: `postgres://academy:${dbPwd}@${dbEp}/forge_fabrik_academy` },
            { name: "REDIS_URL",    value: `redis://${redisEp}:6379` },
            { name: "APP_PORT",     value: String(containerPort) },
            { name: "JWT_SECRET",   value: jwt },
            { name: "RUST_LOG",     value: "server=info,tower_http=info,sqlx=warn" },
        ],
        logConfiguration: {
            logDriver: "awslogs",
            options: {
                "awslogs-group":         lgName,
                "awslogs-region":        region,
                "awslogs-stream-prefix": "server",
            },
        },
    }])
);

const taskDef = new aws.ecs.TaskDefinition(`${appName}-task`, {
    family:                  `${appName}-${env}`,
    cpu:                     "512",
    memory:                  "1024",
    networkMode:             "awsvpc",
    requiresCompatibilities: ["FARGATE"],
    executionRoleArn:        execRole.arn,
    taskRoleArn:             taskRole.arn,
    containerDefinitions:    containerDefs,
    tags: { Environment: env },
});

// ── ECS service ───────────────────────────────────────────────────────────────
new aws.ecs.Service(`${appName}-service`, {
    cluster:        cluster.arn,
    taskDefinition: taskDef.arn,
    desiredCount:   isProd ? 2 : 1,
    launchType:     "FARGATE",
    networkConfiguration: {
        subnets:         vpc.privateSubnetIds,
        securityGroups:  [appSg.id],
        assignPublicIp:  false,
    },
    loadBalancers: [{
        targetGroupArn: tg.arn,
        containerName:  "server",
        containerPort:  containerPort,
    }],
    deploymentCircuitBreaker:  { enable: true, rollback: true },
    tags: { Name: `${appName}-service-${env}`, Environment: env },
}, { dependsOn: [httpListener] });

// ── Stack outputs ─────────────────────────────────────────────────────────────
export const albEndpoint   = pulumi.interpolate`http://${alb.dnsName}`;
export const ecrRepository = repo.repositoryUrl;
export const dbHost        = db.address;
export const redisHost     = redisAddr;
export const ecsCluster    = cluster.name;
export const logGroupName  = logGroup.name;
export const secretsArn    = secretStore.arn;
