import type { Endpoints } from "./schema";

type GetUserEndpoint = Endpoints["get"]["/user"];

const printEndpointMeta = (endpoint: GetUserEndpoint): void => {
  console.log("path:", endpoint.path);
  console.log("method:", endpoint.method);
  console.log("operationId:", endpoint.operationId ?? "-");
  console.log("params:", endpoint.params ?? {});
  console.log("body:", endpoint.body ?? "-");
  console.log("responses:", endpoint.responses ?? {});
};

void printEndpointMeta;
