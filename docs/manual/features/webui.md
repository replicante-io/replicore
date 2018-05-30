# WebUI
Replicante comes with a lightweight web-based user interface.
The main purpose of which is to visualise collected information and events.

The WebUI aims to be minimal and self-explicatory so just install it and click around.


## Install
{% method %}
The WebUI lives in a dedicated repository and is an nodejs application.
It can be installed by fetching the code, installing dependences, building static assets.

{% common %}
```bash
git clone https://github.com/replicante-io/webui.git
cd webui/
npm install
npm run build
```
{% endmethod %}

{% method %}
All the WebUI needs to work now is the address of the replicante API server.
This should be passed in through environment variables.

{% common %}
```bash
export REPLI_BACKEND_ROOT="http://localhost:16016"
npm run server
```
{% endmethod %}
