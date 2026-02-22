/// Shared OpenAPI fixture that exercises anyOf/oneOf/allOf in one document.
pub const COMBINATORS_OPENAPI_JSON: &str = r##"{
  "paths": {
    "/base-pet": {
      "get": {
        "operationId": "getBasePet",
        "responses": {
          "200": {
            "content": {
              "application/json": {
                "schema": { "$ref": "#/components/schemas/BasePet" }
              }
            }
          }
        }
      }
    },
    "/filters": {
      "get": {
        "operationId": "getPetFilter",
        "responses": {
          "200": {
            "content": {
              "application/json": {
                "schema": { "$ref": "#/components/schemas/PetFilter" }
              }
            }
          }
        }
      }
    },
    "/kinds": {
      "get": {
        "operationId": "getCatKind",
        "responses": {
          "200": {
            "content": {
              "application/json": {
                "schema": { "$ref": "#/components/schemas/CatKind" }
              }
            }
          }
        }
      }
    },
    "/pets": {
      "post": {
        "operationId": "createPet",
        "requestBody": {
          "content": {
            "application/json": {
              "schema": { "$ref": "#/components/schemas/CreatePetRequest" }
            }
          }
        },
        "responses": {
          "201": {
            "content": {
              "application/json": {
                "schema": { "$ref": "#/components/schemas/PetDetails" }
              }
            }
          }
        }
      }
    }
  },
  "components": {
    "schemas": {
      "PetFilter": {
        "anyOf": [
          { "type": "string" },
          { "type": "integer" }
        ]
      },
      "CatKind": {
        "oneOf": [
          { "type": "string", "enum": ["cat"] },
          { "type": "string", "enum": ["tiger"] }
        ]
      },
      "BasePet": {
        "type": "object",
        "properties": {
          "id": { "type": "integer" }
        }
      },
      "PetDetails": {
        "allOf": [
          { "$ref": "#/components/schemas/BasePet" },
          {
            "type": "object",
            "required": ["name"],
            "properties": {
              "name": { "type": "string" }
            }
          }
        ]
      },
      "CreatePetRequest": {
        "type": "object",
        "required": ["name"],
        "properties": {
          "name": { "type": "string" },
          "kind": { "$ref": "#/components/schemas/CatKind" },
          "filter": { "$ref": "#/components/schemas/PetFilter" }
        }
      }
    }
  }
}"##;
