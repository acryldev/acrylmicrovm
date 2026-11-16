# Supporting applications

What sits beside the runtime in this directory:

- [`e2e/`](./e2e/README.md): the end-to-end suite that drives the SDKs and the CLI against a running REST server.
- [`secret-server/`](./secret-server/README.md): the versioned-secret encryption package; the HTTP service
  and its deployment are not implemented yet.
- [`libs/`](./libs): generated REST clients for the Box API, and shared Go packages.
- [`box-images/`](./box-images): container images used as box roots.
- [`dex/`](./dex), [`hack/`](./hack), [`scripts/`](./scripts): local development helpers.

Licence texts for the parts of this directory that carry their own are in [`LICENSES/`](./LICENSES); see
[`NOTICE`](./NOTICE) in this directory and the repository `NOTICE`.
