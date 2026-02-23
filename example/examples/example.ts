import type { Endpoints, Schemas } from "../generated/ts/PetstoreV2";

// Schemas
type Pet = Schemas["Pet"];

// Endpoints
type GetPet = Endpoints["get"]["/pet/{petId}"];

void ({} as Pet);
void ({} as GetPet["params"]["path"]);
