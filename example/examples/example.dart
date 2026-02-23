import '../generated/dart/PetstoreV2.dart';

void main() {
  // Schemas
  final petSchema = Schemas.pet;

  // Endpoints
  final getPet = Endpoints.get.petPetId;

  print(petSchema);
  print(getPet.path);
}
