import 'package:flutter/material.dart';
import 'package:path_provider/path_provider.dart';

import 'src/core/bridge/api.dart';
import 'src/core/bridge/frb_generated.dart';
import 'src/core/rpc.gen.dart';
import 'src/core/rpc_models.gen.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const KoiBootstrap());
}

Future<RpcClient> createRpcClient() async {
  final dataDirectory = await getApplicationSupportDirectory();
  await RustLib.init();
  final client = await createClient(dataDir: dataDirectory.path);
  return RpcClient(client);
}

class KoiBootstrap extends StatefulWidget {
  const KoiBootstrap({super.key});

  @override
  State<KoiBootstrap> createState() => _KoiBootstrapState();
}

class _KoiBootstrapState extends State<KoiBootstrap> {
  late Future<RpcClient> rpc = createRpcClient();

  void retry() {
    setState(() {
      rpc = createRpcClient();
    });
  }

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<RpcClient>(
      future: rpc,
      builder: (context, snapshot) {
        if (snapshot.hasError) {
          return MaterialApp(
            title: 'koi',
            theme: koiTheme(),
            home: _StartupError(error: snapshot.error!, onRetry: retry),
          );
        }
        if (!snapshot.hasData) {
          return MaterialApp(
            title: 'koi',
            theme: koiTheme(),
            home: const Scaffold(
              body: Center(child: CircularProgressIndicator()),
            ),
          );
        }
        return KoiApp(rpc: snapshot.requireData);
      },
    );
  }
}

ThemeData koiTheme() => ThemeData(
  colorScheme: ColorScheme.fromSeed(seedColor: Colors.orange),
  scaffoldBackgroundColor: const Color(0xfff7f7f5),
  cardTheme: const CardThemeData(margin: EdgeInsets.zero, elevation: 0),
);

class KoiApp extends StatelessWidget {
  const KoiApp({required this.rpc, super.key});

  final RpcClient rpc;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'koi',
      theme: koiTheme(),
      home: NetworksPage(rpc: rpc),
    );
  }
}

class _StartupError extends StatelessWidget {
  const _StartupError({required this.error, required this.onRetry});

  final Object error;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Center(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                const Icon(Icons.error_outline, size: 48),
                const SizedBox(height: 16),
                const Text(
                  'Koi could not start',
                  style: TextStyle(fontSize: 20, fontWeight: FontWeight.w600),
                ),
                const SizedBox(height: 8),
                SelectableText(
                  error.toString(),
                  textAlign: TextAlign.center,
                  style: const TextStyle(color: Colors.black54),
                ),
                const SizedBox(height: 16),
                FilledButton(
                  onPressed: onRetry,
                  child: const Text('Try again'),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class NetworksPage extends StatefulWidget {
  const NetworksPage({required this.rpc, super.key});

  final RpcClient rpc;

  @override
  State<NetworksPage> createState() => _NetworksPageState();
}

class _NetworksPageState extends State<NetworksPage> {
  late Future<List<Network>> networks = widget.rpc.networkListPresets();

  void retry() {
    setState(() {
      networks = widget.rpc.networkListPresets();
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: Colors.transparent,
        title: const Text('Networks'),
      ),
      body: FutureBuilder<List<Network>>(
        future: networks,
        builder: (context, snapshot) {
          if (snapshot.hasError) {
            return _NetworkError(onRetry: retry);
          }
          if (!snapshot.hasData) {
            return const Center(child: CircularProgressIndicator());
          }
          return DefaultNetworksList(networks: snapshot.requireData);
        },
      ),
    );
  }
}

class DefaultNetworksList extends StatelessWidget {
  const DefaultNetworksList({required this.networks, super.key});

  final List<Network> networks;

  @override
  Widget build(BuildContext context) {
    if (networks.isEmpty) {
      return const Center(child: Text('No default networks available'));
    }

    return ListView.separated(
      padding: const EdgeInsets.fromLTRB(20, 12, 20, 32),
      itemCount: networks.length + 1,
      separatorBuilder: (context, index) => const SizedBox(height: 10),
      itemBuilder: (context, index) {
        if (index == 0) {
          return const Padding(
            padding: EdgeInsets.only(bottom: 10),
            child: Text(
              'Default networks',
              style: TextStyle(fontSize: 14, color: Colors.black54),
            ),
          );
        }

        final network = networks[index - 1];
        return Card(
          clipBehavior: Clip.antiAlias,
          child: ListTile(
            contentPadding: const EdgeInsets.symmetric(
              horizontal: 16,
              vertical: 8,
            ),
            leading: _NetworkIcon(network: network),
            title: Text(
              network.networkName,
              style: const TextStyle(fontWeight: FontWeight.w600),
            ),
            subtitle: Text('Chain ID ${network.networkIdentity}'),
          ),
        );
      },
    );
  }
}

class _NetworkIcon extends StatelessWidget {
  const _NetworkIcon({required this.network});

  final Network network;

  @override
  Widget build(BuildContext context) {
    final iconUrl = network.networkIconUrl;
    final fallback = CircleAvatar(
      child: Text(
        network.networkName.isEmpty
            ? '?'
            : network.networkName.substring(0, 1).toUpperCase(),
      ),
    );

    if (iconUrl == null) {
      return fallback;
    }

    return ClipOval(
      child: Image.network(
        iconUrl,
        width: 40,
        height: 40,
        fit: BoxFit.cover,
        errorBuilder: (context, error, stackTrace) => fallback,
      ),
    );
  }
}

class _NetworkError extends StatelessWidget {
  const _NetworkError({required this.onRetry});

  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.cloud_off_outlined, size: 40),
          const SizedBox(height: 12),
          const Text('Could not load default networks'),
          const SizedBox(height: 12),
          FilledButton(onPressed: onRetry, child: const Text('Try again')),
        ],
      ),
    );
  }
}
