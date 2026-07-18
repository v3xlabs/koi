import 'package:flutter_test/flutter_test.dart';

import 'package:koi/src/core/bridge/api.dart';
import 'package:koi/src/core/bridge/frb_generated.dart';

void main() {
  setUpAll(() async {
    // Loads libkoi_ffi from crates/ffi/target/release — `just mobile-test`
    // builds it and symlinks the workspace target dir there.
    await RustLib.init();
  });

  test('pings through the in-process rust dispatcher', () async {
    final client = await createClient();
    final response = await systemPing(client: client);

    expect(response, 'OK');
  });
}
