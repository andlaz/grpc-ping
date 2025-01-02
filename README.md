Testing and monitoring complex ( or not ) ingress pipelines to a Kubernetes
GRPC service

A clap CLI app that produces a small, statically linked binary and a root-less container image

## Install 

### Configure helm repo

First, add this repo as a helm repo for access to its charts
```bash
helm repo add grpc-ping https://raw.githubusercontent.com/andlaz/grpc-ping/master/charts
```

### Request certificate

if you will be serving TLS from the service ( as opposed to, say, terminating it at ingress and not re-encrypting for internal traffic ),
you might want to request a signed certificate and key pair with `cert-manager`

Otherwise, your ingress controller might already integrate with `cert-manager` to do this for you

```bash
helm repo install grpc-canary-cert grpc-ping/grpc-ping-cert \
  --set hostname=ingress-test.example.com,issuer=letsencrypt-prod,secretName=grpc-canary-tls
```

### Start service

```bash
helm repo install grpc-canary grpc-ping/grpc-ping \
  --set tls.secretName=grpc-canary-tls
```

The above will start the TLS-wrapped service on `$POD_IP:8080`
To start non-TLS, simply omit the `tls.secretName` chart value

### Configure a Route

There are a number of standard and 3rd party ( CR ) ways of routing traffic to a service in Kubernetes.
A contemporary one perhaps is the `gateway.networking.k8s.io/v1` API and we provide a simple chart
to manage a `GRPCRoute` resource

```bash
helm repo install test-route grpc-ping/grpc-ping-grpc-route \
  --set parent=someGateway,hostname=ingress-test.example.com
```

## Test

Use a ( reflection-capable, or carry `proto/ping.proto` around ) GRPC client like [vadimi/grpc-client-cli](https://github.com/vadimi/grpc-client-cli)

