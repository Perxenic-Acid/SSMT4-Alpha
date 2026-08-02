# 如何在Release版本中打开控制台？
需要设置cargo.toml和tauri.conf.json中的devtools字段

# cargo.toml

tauri = { version = "2", features = ["protocol-asset", "devtools"] }

# tauri.conf.json

 "devtools": true,
  "app": {
    "windows": [
      {
        "center": true,
        "title": "SSMT4",
        "width": 1056,
        "height": 594,
        "minWidth": 1056,
        "minHeight": 594,
        "devtools": true,
        "decorations": false,
        "backgroundColor": "#000000",
        "visible": false
      }
    ],