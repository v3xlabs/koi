import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'package:koi/src/core/bridge/api.dart';
import 'package:koi/src/core/bridge/frb_generated.dart';
import 'package:koi/src/core/rpc.gen.dart';

void main() {
  setUpAll(() async {
    // Loads libkoi_ffi from crates/ffi/target/release — `just mobile-test`
    // builds it and symlinks the workspace target dir there.
    await RustLib.init();
  });

  test('lists network presets through the generated RPC client', () async {
    final dataDirectory = await Directory.systemTemp.createTemp('koi-mobile-');
    addTearDown(() => dataDirectory.delete(recursive: true));

    final client = await createClient(dataDir: dataDirectory.path);
    final networks = await RpcClient(client).networkListPresets();

    expect(networks, isNotEmpty);
    expect(networks.first.networkName, isNotEmpty);
    expect(File('${dataDirectory.path}/koi.db').existsSync(), isTrue);
  });
}
