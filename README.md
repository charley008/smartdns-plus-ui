# smartdns-plus-ui

`smartdns-plus-ui` 是一个给 SmartDNS 使用的独立 WebUI 插件。

前提很明确：

- 目标环境已经安装并可正常运行 SmartDNS
- 这里只负责部署 `smartdns-plus-ui` 插件本身
- 不负责安装 SmartDNS 主程序

默认访问地址：

- `http://127.0.0.1:6081/`
- 默认账号：`admin`
- 默认密码：`password`

推荐直接下载 GitHub Release 使用。
会提供两类包：

- `smartdns-plus-ui-runtime.tar.gz`
  适合已经安装好 SmartDNS 的 Linux 主机
- `smartdns-plus-ui-docker-runtime.tar.gz`
  适合 Docker 运行目录

## 非 Docker 部署

前提：

- 目标机器已经安装好 SmartDNS
- 系统里存在 `smartdns` 服务，或至少已有 `/etc/smartdns/`

步骤：

1. 下载并解压 `smartdns-plus-ui-runtime.tar.gz`
2. 进入解压后的 `runtime/` 目录
3. 执行：

```sh
sudo bash ./install.sh
```

这个脚本会自动完成：

- 复制 `dist/smartdns_plus_ui.so` 到 `/usr/lib/smartdns/`
- 复制 `dist/wwwroot/` 到 `/usr/share/smartdns-plus/wwwroot/`
- 初始化 `/etc/smartdns/conf.d/`
- 初始化 `/etc/smartdns/rules/`
- 初始化或更新 `/etc/smartdns/smartdns.conf`
- 备份原有 `/etc/smartdns/smartdns.conf`
- 尝试重启 `smartdns` 服务

默认使用的路径：

```text
/usr/lib/smartdns/smartdns_plus_ui.so
/usr/share/smartdns-plus/wwwroot/
/etc/smartdns/smartdns.conf
/etc/smartdns/conf.d/
/etc/smartdns/rules/
```

如果系统没有自动重启成功，请手动重启 SmartDNS。

## Docker 部署

前提：

- SmartDNS 已经以 Docker 方式部署
- 你只是在现有运行目录里补上 `smartdns-plus-ui`
- 准备一个 SmartDNS 运行目录，例如 `/data/smartdns`

推荐目录结构：

```text
/data/smartdns/
├─ compose.yaml
├─ Dockerfile
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

步骤：

1. 下载并解压 `smartdns-plus-ui-docker-runtime.tar.gz`
2. 把解压后的 `docker/` 内容放到你的运行目录
3. 进入运行目录启动：

```sh
cd /data/smartdns
docker compose up -d --build
```

如果你已经有自己的运行目录，也可以在源码仓库里直接补上插件文件：

```sh
cd /projects/smartdns-plus-ui
./scripts/install-docker-runtime.sh /data/smartdns
```

这个脚本会自动完成：

- 复制 `smartdns_plus_ui.so` 到 `dist/`
- 复制 `wwwroot` 到 `dist/wwwroot/`
- 初始化 `etc/smartdns.conf`
- 初始化 `etc/conf.d/`
- 初始化 `etc/rules/`
- 备份原有 `etc/smartdns.conf`
- 自动生成 `compose.yaml`（如果不存在）

核心挂载如下：

```yaml
volumes:
  - ./etc:/etc/smartdns
  - ./db:/var/lib/smartdns
  - ./log:/var/log/smartdns
```

`smartdns_plus_ui.so` 和 `wwwroot` 会在镜像构建时复制到容器镜像层，避免 NAS 目录的 `noexec` 挂载导致插件加载失败。更新 `dist/` 后请重新执行 `docker compose up -d --build`。

默认模板目录在仓库中：

- [docker/compose.yaml](D:/projects/smartdns-plus-ui/docker/compose.yaml)
- [docker/etc](D:/projects/smartdns-plus-ui/docker/etc)

## 说明

- 这份 README 只讨论“如何给已安装 SmartDNS 增加 `smartdns-plus-ui` 插件”
- WebUI 只接管 `smartdns.conf` 中的插件块和 managed 区块
- 页面配置会写入 `conf.d/*.conf`
- 首次接管已有配置时，会备份原有 `smartdns.conf`
- 如果你已经启用了原版 `smartdns_ui.so`，请避免端口冲突

当前默认端口：

- 原版 WebUI 常见端口：`6080`
- `smartdns-plus-ui` 默认端口：`6081`
