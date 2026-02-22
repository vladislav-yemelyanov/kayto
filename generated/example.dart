import 'schema.dart';

void main() {
  final endpoint = Endpoints.get.user;

  print('path: ${endpoint.path}');
  print('method: ${endpoint.method}');
  print('operationId: ${endpoint.operationId ?? '-'}');
  print('bodyType: ${endpoint.bodyType ?? '-'}');
  print('params: ${endpoint.params ?? {}}');
  print('responses: ${endpoint.responses ?? {}}');
}
