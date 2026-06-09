export default {
  app: { title: 'Eussh' },
  titlebar: { settings: '设置', debug: '调试日志' },
  tabbar: { newTerminal: '新建终端', newFileManager: '文件管理' },
  sidebar: { openServers: '已连接', saved: '已保存', noServers: '无已连接服务器', noSaved: '无已保存服务器', addServer: '添加服务器', delete: '删除' },
  status: { noServer: '未选择服务器', connecting: '连接中...', error: '连接错误', disconnected: '已断开', connected: '已连接', latency: '{ms}ms', uploaded: '↑ {name} 上传成功', downloaded: '↓ {name} 已下载' },
  terminal: { copy: '复制', paste: '粘贴' },
  overview: { title: '服务器总览', openTerminal: '打开终端', fileManager: '文件', cpu: 'CPU', memory: '内存', storage: '磁盘', timezone: '时区', uptime: '运行时间', latency: '延迟', host: '主机', cores: '核' },
  filemanager: { title: '文件管理', connecting: '正在连接 SFTP...', connectionFailed: 'SFTP 连接失败', emptyDir: '空目录', loading: '加载中...', listView: '列表', iconView: '图标', refresh: '刷新', newFolder: '新建文件夹', newFolderPrompt: '新文件夹名称:', renamePrompt: '将 {name} 重命名为:', create: '创建', cancel: '取消', back: '后退', forward: '前进', open: '打开', download: '下载', downloadAsArchive: '下载为压缩包', copy: '复制', cut: '剪切', paste: '粘贴', duplicate: '创建副本', rename: '重命名', delete: '删除', permissions: '权限', permissionsFor: '{name} 的权限', apply: '应用', name: '名称', size: '大小', modified: '修改时间', owner: '所有者', group: '用户组', perms: '权限', chmodHint: '例如 755, 644, 777', upload: '正在上传 {name}', downloadOp: '正在下载 {name}' },
  connection: { newTitle: '新建连接', editTitle: '编辑连接', name: '名称', host: '主机', port: '端口', username: '用户名', auth: '认证方式', password: '密码', privateKey: '私钥', keyPath: '私钥路径', passphrase: '密码短语（可选）', cancel: '取消', add: '添加', save: '保存', nameRequired: '名称为必填项', hostRequired: '主机为必填项', portInvalid: '端口无效', userRequired: '用户名为必填项', pwRequired: '密码为必填项', keyRequired: '私钥路径为必填项' },
  settings: { title: '设置', appearance: '外观', theme: '主题', light: '浅色', dark: '深色', system: '系统', terminalColors: '终端配色', accentColor: '主题色', titlebarStyle: '标题栏样式', titlebarMacos: 'macOS 风格', titlebarWin11: 'Windows 11 风格', statusbarStyle: '底栏样式', statusbarDefault: '默认', statusbarAccent: '主题色', terminal: '终端', fontFamily: '字体', fontSize: '字号', cursor: '光标', block: '方块', bar: '竖线', underline: '下划线', monitoring: '监控', sysRefresh: '系统刷新间隔', pingInterval: '延迟检测间隔', showTraffic: '流量监控', language: '语言' },
  welcome: { heading: 'Eussh', subtitle: 'SSH 客户端', hint: '从侧边栏添加服务器开始使用' },
  toast: { renderError: '渲染错误: {msg}', connectFailed: '连接失败: {msg}', terminalFailed: '打开终端失败: {msg}' },
}
