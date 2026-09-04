import { getLocale, t } from "@/lib/i18n";

const roleNames: Record<string, [string, string]> = {
    owner: ["所有者", "Owner"],
    admin: ["管理员", "Administrator"],
    viewer: ["查看者", "Viewer"],
};

const roleDescriptions: Record<string, [string, string]> = {
    owner: ["内置所有者角色，拥有全部权限。", "Built-in owner role with all permissions."],
    admin: [
        "内置管理员角色，拥有日常管理权限。",
        "Built-in administrator role for daily management.",
    ],
    viewer: ["内置查看者角色，仅拥有只读权限。", "Built-in viewer role with read-only access."],
};

export const localizeBuiltInRoleName = (code: string, fallback: string) => {
    const value = roleNames[code];
    return value ? t(...value) : fallback;
};

export const localizeBuiltInRoleDescription = (code: string, fallback?: string | null) => {
    const value = roleDescriptions[code];
    return value ? t(...value) : (fallback ?? "");
};

export const localizeBuiltInUserName = (
    username: string | undefined,
    fallback: string | null | undefined,
) => (username ? localizeBuiltInRoleName(username, fallback || username) : (fallback ?? ""));

const moduleNames: Record<SystemModule.Id, [string, string]> = {
    monitor: ["监控", "Monitoring"],
    insights: ["分析", "Insights"],
    reports: ["自动化", "Automation"],
};

export const localizeModuleName = (id: SystemModule.Id, fallback: string) =>
    moduleNames[id] ? t(...moduleNames[id]) : fallback;

const moduleMenuNames: Record<string, [string, string]> = {
    "monitor:overview": ["概览", "Overview"],
    "monitor:nodes": ["节点", "Nodes"],
    "monitor:incidents": ["告警事件", "Alert incidents"],
    "monitor:summaries": ["日报", "Daily summaries"],
    "insights:overview": ["概览", "Overview"],
    "insights:details": ["明细", "Details"],
    "reports:templates": ["模板", "Templates"],
    "reports:runs": ["填报执行", "Runs"],
};

export const localizeModuleMenuName = (
    moduleId: SystemModule.Id,
    code: string,
    fallback: string,
) => {
    const value = moduleMenuNames[`${moduleId}:${code}`];
    return value?.includes(fallback) ? t(...value) : fallback;
};

const permissionSegments: Record<string, string> = {
    dashboard: "Dashboard",
    system: "System",
    user: "User",
    role: "Role",
    menu: "Menu",
    module: "Module",
    status: "Status",
    manage: "Management",
    log: "Log",
    task: "Task",
    deploy: "Deployment",
    monitor: "Monitoring",
    node: "Node",
    check: "Service check",
    incident: "Incident",
    settings: "Settings",
    insights: "Insights",
    overview: "Overview",
    project: "Project",
    event: "Event",
    page: "Page",
    api: "API",
    reports: "Automation",
    report: "Report",
    flow: "Flow",
    run: "Run",
    schedule: "Schedule",
    template: "Template",
    create: "Create",
    delete: "Delete",
    list: "List",
    options: "Options",
    update: "Update",
    password: "Password",
    view: "View",
    export: "Export",
    analyze: "Analyze",
    recover: "Recover",
    restart: "Restart",
};

export const localizeBuiltInMenuName = (
    record: Pick<Menu.Item, "code" | "name" | "isSystem" | "moduleId" | "moduleMenuCode">,
) => {
    if (record.moduleId && record.moduleMenuCode) {
        return localizeModuleMenuName(
            record.moduleId as SystemModule.Id,
            record.moduleMenuCode,
            record.name,
        );
    }
    if (record.code === "*") {
        return t("全部权限", "All permissions");
    }
    if (!record.isSystem || getLocale() === "zh-CN") {
        return record.name;
    }
    const segments = record.code.split(":").filter((segment) => segment !== "*");
    const localized = segments.map((segment) => permissionSegments[segment] ?? segment).join(" · ");
    return localized || record.name;
};

const taskText: Record<string, { name: [string, string]; description: [string, string] }> = {
    "cleanup-operation-logs-retention": {
        name: ["清理操作日志", "Clean operation logs"],
        description: [
            "删除超过配置保留天数的操作日志。",
            "Delete operation logs older than the configured retention period.",
        ],
    },
    "cleanup-task-runs-retention": {
        name: ["清理任务记录", "Clean task run records"],
        description: [
            "删除超过配置保留天数的定时任务执行记录。",
            "Delete scheduled task runs older than the configured retention period.",
        ],
    },
    "sqlite-storage-maintenance": {
        name: ["SQLite 存储维护", "SQLite storage maintenance"],
        description: [
            "执行 WAL 检查点、优化 SQLite 查询规划统计并回收可复用页面。",
            "Checkpoint WAL, optimize SQLite planner statistics, and reclaim reusable pages.",
        ],
    },
};

export const localizeBuiltInTaskName = (taskKey: string, fallback: string) => {
    const value = taskText[taskKey];
    return value ? t(...value.name) : fallback;
};

export const localizeBuiltInTaskDescription = (taskKey: string, fallback?: string | null) => {
    const value = taskText[taskKey];
    return value ? t(...value.description) : (fallback ?? "");
};

const apiErrorMessages: Record<number, [string, string]> = {
    10003: ["密码处理失败，请重试。", "Password processing failed. Please try again."],
    10004: ["账号已禁用。", "This account is disabled."],
    10005: ["账号正在等待审核。", "This account is pending approval."],
    10006: ["账号已锁定。", "This account is locked."],
    10007: ["用户状态无效。", "The user status is invalid."],
    10008: ["不能修改内置管理员账号。", "The built-in administrator account cannot be modified."],
    10009: ["不能修改系统内置角色。", "System built-in roles cannot be modified."],
    10010: ["不能修改系统内置菜单。", "System built-in menus cannot be modified."],
    10011: ["当前密码不正确。", "The current password is incorrect."],
    10012: ["两次输入的新密码不一致。", "The new password confirmation does not match."],
    10101: ["用户名或密码错误。", "Invalid username or password."],
    10103: ["登录令牌生成失败，请重试。", "Failed to generate the login token. Please try again."],
    10201: ["用户名已存在。", "The username already exists."],
    10202: ["邮箱已存在。", "The email already exists."],
    20001: [
        "服务暂时不可用，请稍后重试。",
        "The service is temporarily unavailable. Please try again later.",
    ],
    20002: [
        "头像目录创建失败，请稍后重试。",
        "Failed to create the avatar directory. Please try again later.",
    ],
    20003: [
        "头像文件保存失败，请稍后重试。",
        "Failed to save the avatar file. Please try again later.",
    ],
    30000: ["登录已失效，请重新登录。", "Your session has expired. Please sign in again."],
};

export const localizeApiError = (code: number | undefined, fallback: string) => {
    const value = code === undefined ? undefined : apiErrorMessages[code];
    return value ? t(...value) : fallback;
};
