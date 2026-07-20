import 'dart:io';

enum Kind { object, stringEnum, alias, union }

final class Declaration {
  const Declaration(this.name, this.source, this.kind);
  final String name;
  final String source;
  final Kind kind;
}

String lowerCamel(String value) {
  final words = value.split('_').where((word) => word.isNotEmpty).toList();
  if (words.isEmpty) return 'value';
  return words.first.toLowerCase() +
      words
          .skip(1)
          .map((word) => word[0].toUpperCase() + word.substring(1))
          .join();
}

String enumIdentifier(String value) {
  final identifier = lowerCamel(value.replaceAll(RegExp('[^A-Za-z0-9_]'), '_'));
  return RegExp(r'^\d').hasMatch(identifier) ? 'value$identifier' : identifier;
}

List<String> splitTopLevel(String source, String separator) {
  final values = <String>[];
  var angle = 0;
  var brace = 0;
  var start = 0;
  for (var index = 0; index <= source.length - separator.length; index++) {
    switch (source[index]) {
      case '<':
        angle++;
      case '>':
        angle--;
      case '{':
        brace++;
      case '}':
        brace--;
    }
    if (angle == 0 && brace == 0 && source.startsWith(separator, index)) {
      values.add(source.substring(start, index).trim());
      start = index + separator.length;
    }
  }
  values.add(source.substring(start).trim());
  return values.where((value) => value.isNotEmpty).toList();
}

String dartType(String source) {
  final type = source.trim();
  return switch (type) {
    'string' => 'String',
    'number' => 'num',
    'boolean' => 'bool',
    'unknown' => 'Object?',
    'never' => 'Never',
    'null' => 'void',
    _ when type.startsWith('Array<') && type.endsWith('>') =>
      'List<${dartType(type.substring(6, type.length - 1))}>',
    _ when type.startsWith('Record<string, ') && type.endsWith('>') =>
      'Map<String, ${dartType(type.substring(15, type.length - 1))}>',
    _
        when type.contains(' | ') ||
            type.contains(' & ') ||
            type.startsWith('{') =>
      'Map<String, Object?>',
    _ => type,
  };
}

String decode(
  String source,
  String value,
  Map<String, Declaration> declarations,
) {
  final type = source.trim();
  if (type == 'string') return '$value as String';
  if (type == 'number') return '$value as num';
  if (type == 'boolean') return '$value as bool';
  if (type == 'unknown') return value;
  if (type == 'never') return "throw const FormatException('unexpected value')";
  if (type.startsWith('Array<') && type.endsWith('>')) {
    final inner = type.substring(6, type.length - 1);
    return 'decodeRpcList($value, (value) => ${decode(inner, 'value', declarations)})';
  }
  if (type.startsWith('Record<string, ') && type.endsWith('>')) {
    final inner = type.substring(15, type.length - 1);
    if (inner == 'never') return 'const <String, Never>{}';
    return '($value as Map<String, Object?>).map((key, value) => MapEntry(key, ${decode(inner, 'value', declarations)}))';
  }
  final declaration = declarations[type];
  if (declaration?.kind == Kind.object) {
    return '$type.fromJson($value as Map<String, Object?>)';
  }
  if (declaration?.kind == Kind.stringEnum) {
    return '${type}Codec.fromJson($value)';
  }
  if (declaration?.kind == Kind.alias) {
    return decode(declaration!.source, value, declarations);
  }
  return '$value as Map<String, Object?>';
}

String encode(
  String source,
  String value,
  Map<String, Declaration> declarations,
) {
  final type = source.trim();
  if (const {'string', 'number', 'boolean', 'unknown'}.contains(type)) {
    return value;
  }
  if (type.startsWith('Array<') && type.endsWith('>')) {
    final inner = type.substring(6, type.length - 1);
    return '$value.map((value) => ${encode(inner, 'value', declarations)}).toList()';
  }
  if (type.startsWith('Record<string, ') && type.endsWith('>')) {
    final inner = type.substring(15, type.length - 1);
    if (inner == 'never') return 'const <String, Object?>{}';
    return '$value.map((key, value) => MapEntry(key, ${encode(inner, 'value', declarations)}))';
  }
  final declaration = declarations[type];
  if (declaration?.kind == Kind.object) return '$value.toJson()';
  if (declaration?.kind == Kind.stringEnum) return '$value.wireValue';
  if (declaration?.kind == Kind.alias) {
    return encode(declaration!.source, value, declarations);
  }
  return value;
}

void main() {
  final source = File('../web/src/api/bindings.gen.ts').readAsStringSync();
  final pattern = RegExp(r'export type (\w+) = (.*?);\n', dotAll: true);
  final declarations = <String, Declaration>{};
  for (final match in pattern.allMatches(source)) {
    final name = match.group(1)!;
    final body = match
        .group(2)!
        .replaceAll(RegExp(r'/\*\*.*?\*/', dotAll: true), '')
        .trim();
    final alternatives = splitTopLevel(body, ' | ');
    final kind =
        body.startsWith('{') && body.endsWith('}') && !body.contains(' & ')
        ? Kind.object
        : alternatives.isNotEmpty &&
              alternatives.every((value) => value.startsWith('"'))
        ? Kind.stringEnum
        : body.contains(' | ') || body.contains(' & ') || body.startsWith('{')
        ? Kind.union
        : Kind.alias;
    declarations[name] = Declaration(name, body, kind);
  }

  final output = StringBuffer(
    '''// Generated from the Rust RPC contract. Do not edit.
// ignore_for_file: use_null_aware_elements

List<T> decodeRpcList<T>(Object? value, T Function(Object?) decode) =>
    (value as List<Object?>).map(decode).toList();

''',
  );
  for (final declaration in declarations.values) {
    if (declaration.name.endsWith('RpcParams') ||
        declaration.name.endsWith('RpcResult')) {
      continue;
    }
    switch (declaration.kind) {
      case Kind.object:
        final body = declaration.source.substring(
          1,
          declaration.source.length - 1,
        );
        final fields = splitTopLevel(body, ',').map((field) {
          final separator = field.indexOf(':');
          final rawName = field
              .substring(0, separator)
              .trim()
              .replaceAll('"', '');
          final optional = rawName.endsWith('?');
          final wireName = optional
              ? rawName.substring(0, rawName.length - 1)
              : rawName;
          return (
            wireName: wireName,
            name: lowerCamel(wireName),
            sourceType: field.substring(separator + 1).trim(),
            optional: optional,
          );
        }).toList();
        output.writeln('final class ${declaration.name} {');
        output.write('  const ${declaration.name}({');
        for (final field in fields) {
          output.write(
            field.optional
                ? 'this.${field.name}, '
                : 'required this.${field.name}, ',
          );
        }
        output.writeln('});');
        for (final field in fields) {
          output.writeln(
            '  final ${dartType(field.sourceType)}${field.optional ? '?' : ''} ${field.name};',
          );
        }
        output.writeln(
          '  factory ${declaration.name}.fromJson(Map<String, Object?> json) => ${declaration.name}(',
        );
        for (final field in fields) {
          final decoded = decode(
            field.sourceType,
            "json['${field.wireName}']",
            declarations,
          );
          output.writeln(
            '    ${field.name}: ${field.optional ? "json.containsKey('${field.wireName}') ? $decoded : null" : decoded},',
          );
        }
        output.writeln('  );');
        output.writeln('  Map<String, Object?> toJson() => {');
        for (final field in fields) {
          output.writeln(
            field.optional
                ? "    if (${field.name} != null) '${field.wireName}': ${encode(field.sourceType, '${field.name}!', declarations)},"
                : "    '${field.wireName}': ${encode(field.sourceType, field.name, declarations)},",
          );
        }
        output.writeln('  };\n}');
      case Kind.stringEnum:
        output.writeln('enum ${declaration.name} {');
        for (final value in splitTopLevel(declaration.source, ' | ')) {
          final wireValue = value.substring(1, value.length - 1);
          output.writeln("  ${enumIdentifier(wireValue)}('$wireValue'),");
        }
        output.writeln(
          "  ;\n  const ${declaration.name}(this.wireValue);\n  final String wireValue;\n}",
        );
        output.writeln(
          'extension ${declaration.name}Codec on ${declaration.name} {',
        );
        output.writeln(
          '  static ${declaration.name} fromJson(Object? value) => ${declaration.name}.values.singleWhere((entry) => entry.wireValue == value);\n}',
        );
      case Kind.alias:
        output.writeln(
          'typedef ${declaration.name} = ${dartType(declaration.source)};',
        );
      case Kind.union:
        output.writeln('typedef ${declaration.name} = Map<String, Object?>;');
    }
    output.writeln();
  }
  for (final declaration in declarations.values) {
    if (declaration.name.endsWith('RpcParams') ||
        declaration.name.endsWith('RpcResult')) {
      continue;
    }
    output.writeln(
      '${dartType(declaration.name)} decode${declaration.name}(Object? value) => ${decode(declaration.name, 'value', declarations)};',
    );
  }
  File('lib/src/core/rpc_models.gen.dart').writeAsStringSync(output.toString());
}
