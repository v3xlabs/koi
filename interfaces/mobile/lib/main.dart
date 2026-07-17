import 'package:flutter/material.dart';

import 'src/core/bridge/api.dart';
import 'src/core/bridge/frb_generated.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  runApp(const KoiApp());
}

class KoiApp extends StatelessWidget {
  const KoiApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'koi',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.orange),
      ),
      home: const HelloPage(),
    );
  }
}

class HelloPage extends StatelessWidget {
  const HelloPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('koi')),
      body: Center(
        child: Text(
          greet(name: 'koi'),
          style: Theme.of(context).textTheme.headlineSmall,
        ),
      ),
    );
  }
}
