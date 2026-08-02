> [!CAUTION]
> **⚠️ 安全性警告**
>
> SSMT4测试版包含带有驱动利用漏洞的WinDivert版本，以及带有注入行为的Run.exe会100%触发杀软误报（本项目大量使用了微软Undocumented函数以及驱动漏洞利用技术以绕过部分游戏的反作弊，虽然并未存在加壳行为，但是百分百会被误报，请自行注意评估使用风险，本人只能以微薄的开发者身份提供最基础的安全性承诺，目前的闭源版本无法保证其避开任何反病毒工具的审查，如有介意请等待其正在筹备中的后续开源版本）。
> 
> SSMT4-Alpha的开源版本SSMT4，包含其d3d12.dll(ProjectBunny)都正在灰度测试中，其相关功能并未完善仍在频繁的开发修改中，会在合适的时间开源发布。
> 
> 由于其更新机制依赖于Github Release，唯一的安全性保证依赖于.sig签名文件，SSMT4-Alpha不会主动在除Github外其它地方发布，如果在其它平台看到了SSMT4的二进制版本，请注意其均未获得二次发布许可，并且请仔细将其与本仓库对应版本Release中.sig文件对比，以防下载到重打包的危险二次发布版本。

# SSMT4 Alpha测试版  ![GitHub Downloads (all assets, latest release)](https://img.shields.io/github/downloads/StarBobis/SSMT4-Alpha/latest/total?color=blue&label=LatestDownloads)


- 配套Blender插件：https://github.com/StarBobis/TheHerta4
- 更多蓝图特性的Blender插件: https://github.com/xuhuan9102/TheHerta4
- SSMT4文档源代码：https://starbobis.github.io/SSMT4-Documents/
- SSMT4文档阅读地址：https://github.com/StarBobis/SSMT4-Documents

# 附加功能移除说明

SSMT4开源版仅包含了基础的启动器、管理器、模型提取功能，基础功能方便社区共建

其他附加功能为了满足开源社区的合规要求，已从SSMT4中删除，需要的话可以赞助LTS获取：

https://afdian.com/item/ec74ee782b2f11efb5a052540025c377

没有这些附加的功能也不会影响你正常使用SSMT，请放心好了。
