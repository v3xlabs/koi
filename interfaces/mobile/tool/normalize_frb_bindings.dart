import 'dart:io';

const _bindingsPath = 'lib/src/core/bridge/frb_generated.dart';
const _generatedStem = "stem: 'UNKNOWN',";
const _crateStem = "stem: 'koi_ffi',";

void main() {
  final bindings = File(_bindingsPath);
  final source = bindings.readAsStringSync();
  final generatedStemCount = _generatedStem.allMatches(source).length;
  final crateStemCount = _crateStem.allMatches(source).length;

  if (generatedStemCount == 0 && crateStemCount == 1) {
    return;
  }

  if (generatedStemCount == 1 && crateStemCount == 0) {
    bindings.writeAsStringSync(source.replaceFirst(_generatedStem, _crateStem));
    return;
  }

  stderr.writeln(
    'Expected one FRB library stem in $_bindingsPath, found '
    '$generatedStemCount fallback and $crateStemCount crate stems.',
  );
  exitCode = 1;
}
