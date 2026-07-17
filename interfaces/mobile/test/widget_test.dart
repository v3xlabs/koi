import 'package:flutter_test/flutter_test.dart';

import 'package:koi/main.dart';
import 'package:koi/src/core/bridge/frb_generated.dart';

void main() {
  setUpAll(() async {
    // Loads libkoi_ffi from crates/ffi/target/release — `just mobile-test`
    // builds it and symlinks the workspace target dir there.
    await RustLib.init();
  });

  testWidgets('greets from the rust core', (tester) async {
    await tester.pumpWidget(const KoiApp());

    expect(find.textContaining('Hello, koi!'), findsOneWidget);
    expect(find.textContaining('rust core'), findsOneWidget);
  });
}
