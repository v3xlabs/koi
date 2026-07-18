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
      home: const PingPage(),
    );
  }
}

class PingPage extends StatefulWidget {
  const PingPage({super.key});

  @override
  State<PingPage> createState() => _PingPageState();
}

class _PingPageState extends State<PingPage> {
  late final Future<String> _ping = _loadPing();

  Future<String> _loadPing() async {
    final client = await createClient();
    return systemPing(client: client);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('koi')),
      body: Center(
        child: FutureBuilder<String>(
          future: _ping,
          builder: (context, snapshot) {
            if (snapshot.hasError) {
              return const Text('Rust core unavailable');
            }
            if (!snapshot.hasData) {
              return const CircularProgressIndicator();
            }
            return Text(
              snapshot.requireData,
              style: Theme.of(context).textTheme.headlineSmall,
            );
          },
        ),
      ),
    );
  }
}
