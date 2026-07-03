# smartdns-plus-ui

`smartdns-plus-ui` 是一个独立维护的 SmartDNS WebUI 插件项目。

目标不是修改 SmartDNS 核心，而是在不破坏原版 `smartdns_ui.so` 的前提下，提供：

- 独立 WebUI 页面
- 独立 HTTP API
- 可视化编辑 `smartdns.conf`
- 设置向导
- 高级配置页
- `smartdns.conf` 仅作为入口文件维护，不再自动生成主配置备份
- `conf.d/*.conf` 在内容变更时会生成同名 `.backup`，用于单文件回滚
- 保存后请求重启 SmartDNS

## 项目定位

建议直接把项目放在：

```text
D:\projects\smartdns-plus-ui
```

这个目录现在已经按“独立项目”方式整理：

- 不再要求从 `plugin/smartdns-plus-ui` 子目录开发
- 构建时默认使用项目内自带的 `vendor/include`
- 不再默认依赖官方 SmartDNS 源码树中的 `../../src`

## 目录说明

```text
smartdns-plus-ui/
├─ .github/workflows/    GitHub Actions 工作流
├─ docker/               Docker 模板与运行产物布局
├─ src/                 Rust 插件源码
├─ vendor/include/      SmartDNS 头文件副本，用于独立编译
├─ wwwroot/             前端页面
├─ tests/               测试
├─ scripts/             部署脚本
├─ Makefile             构建入口
├─ Cargo.toml           Rust 项目定义
└─ README.md            使用说明
```

## 编译

在 WSL2 / Linux 中进入项目目录：

```sh
cd /projects/smartdns-plus-ui
make
```

输出文件：

- `target/smartdns_plus_ui.so`

说明：

- 日常构建不需要官方 SmartDNS 源码目录
- `cargo test` 如果要跑和 SmartDNS 测试库耦合的测试，需要额外提供 `SMARTDNS_TEST_LIB`

## SmartDNS 配置

在 `smartdns.conf` 中加入：

```conf
plugin smartdns_plus_ui.so
smartdns-plus-ui.ip http://0.0.0.0:6081
smartdns-plus-ui.www-root /usr/share/smartdns-plus/wwwroot
smartdns-plus-ui.config-file /etc/smartdns/smartdns.conf
smartdns-plus-ui.rules-dir /etc/smartdns/rules
smartdns-plus-ui.token-expire 600
smartdns-plus-ui.user admin
smartdns-plus-ui.password password
smartdns-plus-ui.enable-terminal yes
```

如果你已经启用了原版 WebUI：

```conf
plugin smartdns_ui.so
smartdns-ui.ip http://0.0.0.0:6080
...
```

那么两者可以并存，但端口不能冲突。推荐：

- 原版保留 `6080`
- `smartdns-plus-ui` 使用 `6081`

## 部署方式

这个项目需要同时覆盖两种场景：

- Docker 中运行的 SmartDNS
- 宿主机直接运行的 SmartDNS

核心原则只有一条：

- `smartdns_plus_ui.so` 和 `wwwroot` 必须进入 SmartDNS 实际运行环境

也就是说：

- Docker 部署时，通过 volume 把 `.so` 和 `wwwroot` 挂进容器
- 非 Docker 部署时，把 `.so` 和 `wwwroot` 直接安装到系统路径

### Docker 部署

推荐使用一个自包含运行目录，例如：

```text
/data/smartdns/
├─ compose.yaml
├─ dist/
│  ├─ smartdns_plus_ui.so
│  └─ wwwroot/
├─ etc/
│  ├─ smartdns.conf
│  ├─ conf.d/
│  └─ rules/
├─ db/
└─ log/
```

这样运行目录本身就是完整可迁移的，不依赖你的源码路径。

仓库内的 Docker 模板位于：

- [docker/compose.yaml](D:/projects/smartdns-plus-ui/docker/compose.yaml)
- [docker/etc/smartdns.conf](D:/projects/smartdns-plus-ui/docker/etc/smartdns.conf)
- [docker/etc/conf.d](D:/projects/smartdns-plus-ui/docker/etc/conf.d)
- [docker/etc/rules](D:/projects/smartdns-plus-ui/docker/etc/rules)

核心挂载：

```yaml
volumes:
  - ./etc:/etc/smartdns
  - ./db:/var/lib/smartdns
  - ./log:/var/log/smartdns
  - ./dist/smartdns_plus_ui.so:/usr/lib/smartdns/smartdns_plus_ui.so:ro
  - ./dist/wwwroot:/usr/share/smartdns-plus/wwwroot:ro
```

如果你使用 `host network`：

```yaml
network_mode: host
```

如果你使用桥接网络，则需要至少映射：

```yaml
ports:
  - 6081:6081/tcp
```

如果原版 `smartdns_ui.so` 也在同一容器中运行，还可以额外映射：

```yaml
ports:
  - 6080:6080/tcp
  - 6081:6081/tcp
```

Docker 首次安装：

```sh
cd /projects/smartdns-plus-ui
./scripts/install-docker-runtime.sh /data/smartdns
```

这个脚本会：

1. 编译插件
2. 复制 `smartdns_plus_ui.so` 到运行目录的 `dist/`
3. 复制 `wwwroot/` 到运行目录的 `dist/`
4. 创建 `etc/conf.d`、`etc/rules`、`db`、`log`
5. 如果目标环境已经有 `etc/smartdns.conf`，保留原文件内容，只插入插件配置块和 `conf-file` 托管块
6. 如果目标环境还没有 `etc/smartdns.conf`，创建一个最小可用入口文件
   这个入口文件来自仓库内的 `docker/etc/smartdns.conf`
7. 如果目标环境还没有 `etc/conf.d/*.conf`，则从仓库内的 `docker/etc/conf.d/` 复制一套默认模板
8. 如果目标环境还没有 `etc/rules/*.txt`，则从仓库内的 `docker/etc/rules/` 复制默认规则文件
9. 首次接管已有 `smartdns.conf` 时，额外生成一份 `smartdns.conf.pre-smartdns-plus-ui.backup`
10. 如果不存在 Compose 文件，默认生成 `compose.yaml`

如果别人已经在使用 SmartDNS，这个脚本的原则是：

- 不覆盖原有 `smartdns.conf`
- 只在原文件中插入 `smartdns-plus-ui` 插件块
- 只在原文件中插入 `# BEGIN smartdns-plus-ui managed` 托管区块
- 以后 WebUI 只维护 `conf.d/*.conf`
- 用户原本手写在 `smartdns.conf` 里的内容仍然保留

需要注意：

- 如果用户原来的 `smartdns.conf` 已经手写了和 WebUI 相同的配置项，后续要自行清理重复项
- 例如原文件里已有 `server-name`、`cache-size`、`proxy-server`、`nameserver`、`domain-set` 等，而 WebUI 也开始维护这些项时，最终应只保留一份来源

### 非 Docker 部署

如果 SmartDNS 是直接运行在系统里，而不是容器里，推荐安装到这些标准路径：

```text
/usr/lib/smartdns/smartdns_plus_ui.so
/usr/share/smartdns-plus/wwwroot/
/etc/smartdns/smartdns.conf
/etc/smartdns/conf.d/
/etc/smartdns/rules/
```

直接安装：

```sh
cd /projects/smartdns-plus-ui
sudo ./scripts/install-system.sh
```

这个脚本会：

1. 编译插件
2. 把 `.so` 安装到 `/usr/lib/smartdns/`
3. 把 `wwwroot` 安装到 `/usr/share/smartdns-plus/wwwroot/`
4. 创建 `/etc/smartdns/conf.d/` 和 `/etc/smartdns/rules/`
5. 如果 `/etc/smartdns/conf.d/*.conf` 不存在，则用 `docker/etc/conf.d/` 初始化一套默认模板
6. 如果 `/etc/smartdns/rules/*.txt` 不存在，则用 `docker/etc/rules/` 初始化默认规则文件
7. 如果 `/etc/smartdns/smartdns.conf` 不存在，则用 `docker/etc/smartdns.conf` 初始化
8. 在 `/etc/smartdns/smartdns.conf` 中插入插件块和 managed 托管块
9. 如果原有 `smartdns.conf` 已存在，先保存一份 `smartdns.conf.pre-smartdns-plus-ui.backup`

安装完成后，需要你自己按发行版方式重启 SmartDNS 服务。

### 开发调试更新

如果你当前的测试运行目录就是：

```text
/data/smartdns
```

可以直接执行：

```sh
cd /projects/smartdns-plus-ui
./scripts/deploy-to-data-smartdns.sh
```

这个脚本会：

1. 检查 `/data/smartdns` 是否已经是一个已初始化运行目录
2. 自动识别其中的 Compose 文件
3. 编译插件
4. 同步 `dist/smartdns_plus_ui.so`
5. 同步 `dist/wwwroot/`
6. 重启对应运行目录下的 Docker Compose

如果你的运行目录不是 `/data/smartdns`，也可以显式传参：

```sh
cd /projects/smartdns-plus-ui
./scripts/deploy-to-data-smartdns.sh /your/runtime/dir
```

### GitHub Actions 产物

仓库内置了工作流：

- [.github/workflows/build-runtime-assets.yml](D:/projects/smartdns-plus-ui/.github/workflows/build-runtime-assets.yml)

它会自动生成两类产物：

- `smartdns-plus-ui-runtime`
- `smartdns-plus-ui-docker-runtime`

其中通用运行包会包含：

```text
runtime/
├─ compose.yaml
├─ db/
├─ dist/
│  ├─ smartdns_plus_ui.so
│  └─ wwwroot/
├─ log/
└─ etc/
   ├─ smartdns.conf
   ├─ conf.d/
   └─ rules/
```

其中 Docker 运行包会包含：

```text
docker/
├─ compose.yaml
├─ db/
├─ log/
├─ etc/
│  ├─ smartdns.conf
│  ├─ conf.d/
│  └─ rules/
└─ dist/
   ├─ smartdns_plus_ui.so
   └─ wwwroot/
```

如果你想在本地直接生成同结构发布包，也可以执行：

```sh
cd /projects/smartdns-plus-ui
./scripts/package-release.sh
```

它会在项目根目录生成：

- `smartdns-plus-ui-runtime.tar.gz`
- `smartdns-plus-ui-docker-runtime.tar.gz`

这样别人从 GitHub Actions 或 Release 下载后，解压出来就能直接作为 Docker 运行目录的一部分使用，不需要本地再编译 `.so`。

## 配置写入方式

图形界面保存时会：

- 保持主配置 `smartdns.conf` 只作为入口文件
- 在 `# BEGIN smartdns-plus-ui managed` 与 `# END smartdns-plus-ui managed` 之间，按页面引入 `conf.d/*.conf`
- 每个侧边栏页面只负责读写自己对应的一个 `.conf` 文件，避免跨页混写

当前页面与文件映射如下：

```text
/etc/smartdns/conf.d/
├─ 10-basic.conf         基础设置
├─ 20-upstreams.conf     上游 DNS
├─ 30-cache.conf         缓存与性能
├─ 60-sets.conf          集合管理
├─ 40-nameserver.conf    分流设置
├─ 50-rules.conf         规则
├─ 70-logging.conf       日志与审计
└─ 80-network.conf       高级网络
```

说明：

- 文件命名顺序整体以侧边栏页面为准
- 但 `60-sets.conf` 会在主配置中先于 `40-nameserver.conf` 和 `50-rules.conf` 被引入
- 原因是 `domain-set` / `ip-set` 必须先定义，后续 `nameserver`、`domain-rules`、`address`、`cname`、`ip-rules` 才能引用

建议遵守这条原则：

- 某个配置项属于哪个侧边栏页面，就永远写在那个页面对应的 `.conf` 文件里

例如：

- `proxy-server` 只放在 `10-basic.conf`
- 上游的 `-subnet` 只放在 `20-upstreams.conf`
- 全局 `edns-client-subnet` 只放在 `80-network.conf`
- `nameserver` 只放在 `40-nameserver.conf`
- `domain-set` / `ip-set` 只放在 `60-sets.conf`

1. 读取主配置文件
2. 在主配置文件中维护插件块和 managed include 区域
3. 把图形化设置分别写入独立 `.conf` 文件
4. 分文件内容变化时，只为对应文件生成一份 `.backup`

当前托管区块：

```conf
# BEGIN smartdns-plus-ui managed
conf-file /etc/smartdns/conf.d/10-basic.conf
conf-file /etc/smartdns/conf.d/20-upstreams.conf
conf-file /etc/smartdns/conf.d/30-cache.conf
conf-file /etc/smartdns/conf.d/60-sets.conf
conf-file /etc/smartdns/conf.d/40-nameserver.conf
conf-file /etc/smartdns/conf.d/50-rules.conf
conf-file /etc/smartdns/conf.d/70-logging.conf
conf-file /etc/smartdns/conf.d/80-network.conf
# END smartdns-plus-ui managed
```

这样能尽量避免直接覆盖用户原本整份 `smartdns.conf`。

## 当前边界

- 已实现基础校验，但还不是 SmartDNS 官方级全量语法校验
- 保存并重启目前走 `smartdns_restart()`，不是热 reload
- 上游 DNS 已开始结构化，但高级规则编辑页仍在继续补齐
