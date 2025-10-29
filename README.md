Haproxy gRPC agent
------------------

This projects is a haproxy agent (see [Agent checks](https://www.haproxy.com/documentation/haproxy-configuration-tutorials/reliability/health-checks/#agent-checks)) that checks gRPC backend using standard `grpc.health.v1.Health` service method `Check`.

Motivation for project is that standard HTTP checks (`option httpchk`) with netty-based application lead to exceptions like "INTERNAL: Encountered end-of-stream mid-frame".

# Usage

Start agent like:
```
haproxy_grpc_agent <port>
```

Program listens to given port for commands like
```
<backend_server_name> <backend_server_port> <ssl_flag> <proxy_host_name>\n
```
and returns `up` for successful check and `down` for failed check.

In haproxy config backend should be configured like
```
server server1 <backend_server_name>:<backend_server_port> check agent-check agent-inter 5s agent-port <agent_port> agent-addr <agent_host> agent-send "<backend_server_name> <backend_server_port> <ssl_flag> <proxy_host_name>\n"
```
