import { $ } from "bun";

const services = [
  {
    name: "PetstoreV2",
    input: "https://petstore.swagger.io/v2/swagger.json",
  },
  {
    name: "GitHub",
    input: "https://api.apis.guru/v2/specs/github.com/1.1.4/openapi.json",
  },
];

await $`mkdir -p ./generated/ts ./generated/dart`;

for (const service of services) {
  const promise1 = $`kayto --lang ts --input "${service.input}" --output "./generated/ts/${service.name}.ts"`;

  await promise1;

  console.log("");
  console.log("--------------");
  console.log("");

  const promise2 = $`kayto --lang dart --input "${service.input}" --output "./generated/dart/${service.name}.dart"`;

  await promise2;

  console.log("");
  console.log("--------------");
  console.log("");
}

console.log("✅ Done!");
