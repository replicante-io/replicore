---
id: features-webui
title: WebUI
sidebar_label: WebUI
---

Replicante comes with a lightweight web-based user interface.
The main purpose of which is to visualise collected information and events.

The WebUI aims to be minimal and self-explicatory so just install it and click around.


## Install
The WebUI lives in a dedicated repository and is an nodejs application.
It can be installed by fetching the code, installing dependences, building static assets.

```bash
git clone https://github.com/replicante-io/webui.git
cd webui/
npm install
npm run build
```

All the WebUI needs to work now is the address of the replicante API server.
This should be passed in through environment variables.

```bash
export REPLI_BACKEND_ROOT="http://localhost:16016"
npm run server
```
