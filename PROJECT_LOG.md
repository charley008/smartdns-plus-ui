## [2026-07-02 16:10]

### Goal
阅读新版本 SmartDNS Plus UI 交接文档与前端代码，修复首批前端交互问题。

### Findings
- `wwwroot/app.js` 混入了旧版统一规则编辑逻辑，与当前多页面 UI 结构不一致。
- 上游分流新增仍引用已不存在的旧表单元素，导致 `Cannot read properties of null (reading 'value')`。
- 规则页的 `domain-rules` 与 `address/cname` 删除仅修改局部数组，未刷新 UI、未触发保存，也未记录主配置来源规则的删除状态。

### Actions
- 阅读 `HANDOFF-2026-07-02.md`、`wwwroot/index.html`、`wwwroot/app.js`、`src/config_ui.rs`。
- 将上游分流新增逻辑改为仅处理 `nameserver`，移除旧表单依赖。
- 为路由类规则增加统一的合并、删除标记、删除恢复辅助函数。
- 修复规则页删除后无反应的问题，并让删除主配置来源规则后不会在预览或刷新时重新出现。
- 使用 `node --check wwwroot/app.js` 做语法校验。

### Modified Files
- wwwroot/app.js: 修复上游分流新增报错与规则删除同步逻辑
- PROJECT_LOG.md: 记录本次接手与修复过程

### Result
上游分流新增不再访问空 DOM 节点；规则页删除现在会立即刷新并触发自动保存，主配置来源规则的删除也会被正确记入前端状态。

### Next
- 在浏览器里实际回归测试上游分流、规则页删除、预览配置内容是否一致。
- 继续排查其余残留问题，例如编辑逻辑、集合管理、conf-file 读写回填。

## [2026-07-02 16:32]

### Goal
修正运行环境中 `smartdns-working.conf` 与 `smartdns.conf` 的历史错误分流写法。

### Findings
- 官方 `DNS分流` 文档要求后缀分流使用 `nameserver /com/group` 这类格式。
- 当前环境里存在历史错误写法：
  - `nameserver /.cn/cn`
  - `nameserver /com//cn`

### Actions
- 在 WSL2 中备份：
  - `/data/smartdns/etc/smartdns-working.conf.codex-bak-20260702-1`
  - `/data/smartdns/etc/smartdns.conf.codex-bak-20260702-1`
- 将两份配置中的错误行修正为官方格式。
- 重新检查 `nameserver` 行确认结果。

### Modified Files
- /data/smartdns/etc/smartdns-working.conf: 修正后缀分流历史错误写法
- /data/smartdns/etc/smartdns.conf: 修正后缀分流历史错误写法
- PROJECT_LOG.md: 记录本次环境修复

### Result
- `smartdns-working.conf` 现为：
  - `nameserver /cn/cn`
  - `nameserver /com/cn`
- `smartdns.conf` 现为：
  - `nameserver /cn/cn`

### Next
- 继续通过前端新增/删除分流，确认后续不会再次生成 `/.cn/cn` 或 `/com//cn`。
- 如有必要，给加载逻辑增加对旧错误格式的自动兼容迁移。

## [2026-07-02 17:05]

### Goal
将当前测试环境的 `smartdns.conf` 调整为“官方完整模板 + plus-ui include 文件”结构。

### Findings
- 官方完整模板只有少数默认活动项（如 `bind [::]:53`、`log-level info`），其余均为注释示例。
- 当前环境配置适合拆分为：
  - `plus-ui-basic.conf`
  - `plus-ui-upstreams.conf`
  - `plus-ui-rules.conf`
- 仅调整配置文件还不够，现有 WebUI 后端仍然沿用 `smartdns-working.conf -> smartdns.conf` 的覆盖式保存逻辑，后续需要代码迁移。

### Actions
- 拉取官方 `smartdns.conf` 模板到临时文件。
- 备份现有主配置：
  - `/data/smartdns/etc/smartdns.conf.codex-bak-20260702-official-template`
- 生成新的主配置：
  - 保留官方完整注释结构
  - 插入 `# BEGIN smartdns-plus-ui managed` include 区块
  - 只保留 `data-dir` 与 `smartdns_plus_ui.so` 插件配置为活动项
- 生成 3 个 plus-ui include 文件并写入当前实际使用的设置。
- 清理主配置中残留的官方默认活动项，确保不会和 plus-ui include 重复。

### Modified Files
- /data/smartdns/etc/smartdns.conf: 改为官方模板 + plus-ui include 结构
- /data/smartdns/etc/rules/plus-ui-basic.conf: 承载基础设置、缓存、日志等
- /data/smartdns/etc/rules/plus-ui-upstreams.conf: 承载上游 DNS 与代理
- /data/smartdns/etc/rules/plus-ui-rules.conf: 承载域名集合、分流与规则
- PROJECT_LOG.md: 记录本次架构落地

### Result
- 主 `smartdns.conf` 已不再直接承载 WebUI 管理的业务配置，只有：
  - `conf-file /etc/smartdns/rules/plus-ui-basic.conf`
  - `conf-file /etc/smartdns/rules/plus-ui-upstreams.conf`
  - `conf-file /etc/smartdns/rules/plus-ui-rules.conf`
  - `data-dir`
  - `smartdns_plus_ui.so` 插件配置
- WebUI 当前所需的配置已拆分到 3 个 include 文件中。

### Next
- 将后端 `save/apply/preview` 逻辑从 `smartdns-working.conf` 架构迁移到 include-file 架构。
- 迁移完成前，继续使用当前 WebUI 的保存/应用功能可能会把主配置覆盖回旧模式，需要先改代码再继续验证。

## [2026-07-02 17:32]

### Goal
将后端与前端正式迁移到 `plus-ui-basic.conf / plus-ui-upstreams.conf / plus-ui-rules.conf` 的 include-file 架构。

### Findings
- 旧逻辑中：
  - `save_beginner_config()` 会生成整份 `smartdns-working.conf`
  - `apply_working_config()` 会整份覆盖主 `smartdns.conf`
  - `ensure_managed_block()` 还会额外剥离主配置中的大量指令
- 这些行为都和“官方主模板 + plus-ui include”新架构冲突。

### Actions
- 修改 `src/config_ui.rs` 中的保存逻辑：
  - `save_beginner_config()` 改为直接写 3 个 include 文件
  - 保存时只确保主配置含有 managed block，不再生成 working file
- 修改 `ensure_managed_block()`：
  - 只维护 `# BEGIN/END smartdns-plus-ui managed` 区块
  - 不再剥离主配置中的其他内容
- 修改 `apply_working_config()`：
  - 改为“确保主配置区块存在”而非覆盖整份主配置
- 修改 `src/http_server_api.rs`：
  - `/api/plus/apply` 传入 `rules_dir`
- 修改前端预览文案：
  - 不再强调 `smartdns-working.conf`
  - 改为 `smartdns.conf` 与 plus-ui include 合成预览
- 增加预览合成逻辑：
  - 用主配置文本内联 3 个 plus-ui include 文件内容
- 运行：
  - `node --check wwwroot/app.js`
  - `wsl cargo check`

### Modified Files
- src/config_ui.rs: 迁移到 include-file 保存/应用/预览模型
- src/http_server_api.rs: 更新 apply 调用参数
- wwwroot/app.js: 预览页说明文案调整
- wwwroot/index.html: 预览页提示文案调整
- PROJECT_LOG.md: 记录本次代码迁移

### Result
- WebUI 现在不再依赖 `smartdns-working.conf`
- 保存会直接更新：
  - `/etc/smartdns/rules/plus-ui-basic.conf`
  - `/etc/smartdns/rules/plus-ui-upstreams.conf`
  - `/etc/smartdns/rules/plus-ui-rules.conf`
- 应用时不会整份覆盖官方主模板 `smartdns.conf`
- 编译与前端语法检查通过

### Next
- 在浏览器里实际验证：
  - 修改基础设置 / 上游 / 分流 / 规则后，3 个 include 文件是否即时更新
  - 点击“保存并重启”后主配置区块是否保持不变
  - 配置预览是否符合预期

## [2026-07-13 11:05]

### Goal
梳理查询日志的审计能力，并记录后续“查询追踪”功能的设计边界。

### Findings
- 查询日志 API 的 SQLite `domain` 表保存请求摘要，不保存最终 DNS Answer 或实际参与的上游。
- `smartdns.log` 与 `smartdns-audit.log` 都能包含部分返回 IP；但文本日志轮转且不适合作为结构化追踪数据源。
- SmartDNS 的选优过程可能包含多个上游与多个候选 IP，因此“使用哪个上游”不能简化为单值字段。

### Actions
- 新增 `TODO.md`，定义查询追踪的数据范围、存储期限、实施阶段和验收条件。
- 暂不修改 SmartDNS 核心、插件 API 或数据库结构。

### Modified Files
- TODO.md: 记录查询追踪、日志审计和查询日志可读性的待办设计
- PROJECT_LOG.md: 记录本次设计结论

### Result
后续功能将以短期、结构化的“查询追踪”实现为目标；普通查询日志继续保留为长期摘要索引。

### Next
- 继续收集和定位现有 Web UI 功能问题。
- 查询追踪进入实施前，先确认 SmartDNS 核心可提供的最小响应与选优回调。
