import 'dart:io';

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
    final dataDirectory = await Directory.systemTemp.createTemp('koi-mobile-');
    addTearDown(() => dataDirectory.delete(recursive: true));

    final client = await createClient(dataDir: dataDirectory.path);
    final response = await systemPing(client: client);

    expect(response, 'OK');
    expect(File('${dataDirectory.path}/koi.db').existsSync(), isTrue);
  });
}
