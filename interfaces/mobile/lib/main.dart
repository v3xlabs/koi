import 'package:flutter/material.dart';
import 'package:path_provider/path_provider.dart';

import 'src/core/bridge/api.dart';
import 'src/core/bridge/frb_generated.dart';
import 'src/core/rpc.gen.dart';
import 'src/core/rpc_models.gen.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final dataDirectory = await getApplicationSupportDirectory();
  await RustLib.init();
  runApp(KoiApp(dataDirectory: dataDirectory.path));
}

class KoiApp extends StatelessWidget {
  const KoiApp({required this.dataDirectory, super.key});

  final String dataDirectory;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'koi',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.orange),
      ),
      home: PingPage(dataDirectory: dataDirectory),
    );
  }
}

class PingPage extends StatefulWidget {
  const PingPage({required this.dataDirectory, super.key});

  final String dataDirectory;

  @override
  State<PingPage> createState() => _PingPageState();
}

class _PingPageState extends State<PingPage> {
  late final Future<List<Network>> _networks = _loadNetworks();

  Future<List<Network>> _loadNetworks() async {
    final client = await createClient(dataDir: widget.dataDirectory);
    return RpcClient(client).networkListPresets();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('koi')),
      body: Center(
        child: FutureBuilder<List<Network>>(
          future: _networks,
          builder: (context, snapshot) {
            if (snapshot.hasError) {
              return const Text('Rust core unavailable');
            }
            if (!snapshot.hasData) {
              return const CircularProgressIndicator();
            }
            return ListView(
              children: [
                for (final network in snapshot.requireData)
                  ListTile(
                    title: Text(network.networkName),
                    subtitle: Text('Chain ${network.networkIdentity}'),
                  ),
              ],
            );
          },
        ),
      ),
    );
  }
}
