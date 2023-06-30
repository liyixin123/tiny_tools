# 提升工作效率小工具
## SVN 地址转换
1. 修改SVN权限相关时，申请给的是带IP地址的，需要去掉IP地址

2. `0.1.1` 版本开始支持远程修改，依赖 [subversion-edge-modify-tool](https://gitee.com/liyixin123/subversion-edge-modify-tool),该库未正式发布，需要下载到本地配置
> 注意：如果需要自动获取远程路径，需要配置用户和密码
> * 在程序安装路径，新建一个 .env 文件，内容如下：
> ```angular2html
>  USERNAME=用户名
>  PASSWORD=密码
> ```
