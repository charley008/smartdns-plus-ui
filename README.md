# smartdns-plus-ui

`smartdns-plus-ui` 是一个独立维护的 SmartDNS WebUI 插件项目。

目标不是修改 SmartDNS 核心，而是在不破坏原版 `smartdns_ui.so` 的前提下，提供：

- 独立 WebUI 页面
- 独立 HTTP API
- 可视化编辑 `smartdns.conf`
- 设置向导
- 高级配置页
- 保存前自动备份
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
cd /mnt/d/projects/smartdns-plus-ui
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

## Docker 使用方法

### 推荐原则

推荐把 `.so` 放在 Linux 文件系统中，再挂载到容器。

原因：

- `wwwroot` 从 Windows 路径挂载通常没问题
- `.so` 直接从 Windows 路径挂载到容器，常见会遇到 `Permission denied` 或 `dlopen` 问题

所以推荐这样拆开：

1. `.so` 放在 Linux 路径

```text
/data/smartdns/smartdns_plus_ui.so
```

2. `wwwroot` 直接挂载开发目录

```text
/mnt/d/projects/smartdns-plus-ui/wwwroot
```

这样前端页面改完后，只需要重启容器，不需要重复复制 `.so`。

### 方案 A：`host network`

适合你现在这类本地测试环境。

`docker-compose.yml` 示例见：

- [docker-compose.smartdns-example.yml](D:/projects/smartdns-plus-ui/docker-compose.smartdns-example.yml)

核心挂载：

```yaml
volumes:
  - ./etc:/etc/smartdns
  - ./db:/var/lib/smartdns
  - ./log:/var/log/smartdns
  - /data/smartdns/smartdns_plus_ui.so:/usr/lib/smartdns/smartdns_plus_ui.so:ro
  - /mnt/d/projects/smartdns-plus-ui/wwwroot:/usr/share/smartdns-plus/wwwroot:ro
network_mode: host
```

访问地址：

- [http://127.0.0.1:6081/](http://127.0.0.1:6081/)

### 方案 B：桥接网络 `ports`

如果你不用 `host network`，那就必须把 `6081` 映射出来：

```yaml
ports:
  - 6080:6080/tcp
  - 6081:6081/tcp
```

注意：

- 只映射 `6080` 时，原版 WebUI 能访问，但 `smartdns-plus-ui` 的 `6081` 访问不到
- 容器内不存在 `/usr/share/smartdns-plus` 目录通常不是问题，bind mount 时会把目标路径挂出来
- 真正容易出问题的是 `.so` 从 Windows 盘直接挂载，通常不是 `wwwroot`

## 适合你当前环境的一套步骤

你的运行目录如果是：

```text
/data/smartdns
```

建议这样做：

1. 编译插件

```sh
cd /mnt/d/projects/smartdns-plus-ui
make
```

2. 复制 `.so`

```sh
cp /mnt/d/projects/smartdns-plus-ui/target/smartdns_plus_ui.so /data/smartdns/smartdns_plus_ui.so
```

3. 在 `/data/smartdns/etc/smartdns.conf` 中加入插件配置

4. 在 `/data/smartdns/docker-compose.yml` 中加入两个挂载

```yaml
- /data/smartdns/smartdns_plus_ui.so:/usr/lib/smartdns/smartdns_plus_ui.so:ro
- /mnt/d/projects/smartdns-plus-ui/wwwroot:/usr/share/smartdns-plus/wwwroot:ro
```

5. 如果不是 `host network`，再额外映射 `6081:6081`

6. 重启容器

```sh
cd /data/smartdns
sudo docker compose up -d
```

## 一键同步到当前 Docker 环境

如果你当前 SmartDNS Docker 工作目录就是：

```text
/data/smartdns
```

可以直接执行：

```sh
cd /mnt/d/projects/smartdns-plus-ui
./scripts/deploy-to-data-smartdns.sh
```

这个脚本会：

1. 编译插件
2. 把 `target/smartdns_plus_ui.so` 复制到 `/data/smartdns/smartdns_plus_ui.so`
3. 重启 `/data/smartdns` 下的 Docker Compose

## 配置写入方式

图形界面保存时会：

1. 读取主配置文件
2. 自动备份主配置文件
3. 在主配置文件中维护一段 include 区域
4. 把图形化设置分别写入独立规则文件

当前托管区块：

```conf
# BEGIN smartdns-plus-ui managed includes
conf-file /etc/smartdns/rules/plus-ui-basic.conf
conf-file /etc/smartdns/rules/plus-ui-upstreams.conf
conf-file /etc/smartdns/rules/plus-ui-rules.conf
# END smartdns-plus-ui managed includes
```

这样能尽量避免直接覆盖用户原本整份 `smartdns.conf`。

## 当前边界

- 已实现基础校验，但还不是 SmartDNS 官方级全量语法校验
- 保存并重启目前走 `smartdns_restart()`，不是热 reload
- 上游 DNS 已开始结构化，但高级规则编辑页仍在继续补齐
