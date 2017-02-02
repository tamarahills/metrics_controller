var Metrics = require('cd-metrics');

  var logger = function() {
    var args = Array.from(arguments);
    process.stdout.write(args.join(' ') + '\n');
  };

  var clientId = 222219191919;
  var options = {
      locale: 'locale',
      os: 'os',
      osVersion: 'osVersion',
      device: 'device',
      appName: 'appName',
      appVersion: 'appVersion',
      appUpdateChannel: 'app_update_channel',
      appPlatform: 'appPlatform',
      arch: 'arch',
      logger: logger
  };

  process.stdout.write("Instantiating Metrics object\n");
  var metrics = new Metrics(clientId, options);

  process.stdout.write("Recording events...\n");
  metrics.recordEventAsync("category", "action", "label", 987654321);
  metrics.recordEventAsync("category", "action", "label", 987654322, 'client id');
  metrics.recordFloatingPointEventAsync("category", "action", "label", 999999.1);
  metrics.recordFloatingPointEventAsync("category", "action", "label", 999999.2, 'client id');
  metrics.recordEvent("category", "action", "label", 987654323);
  metrics.recordEvent("category", "action", "label", 987654324, 'client id');
  metrics.recordFloatingPointEvent("category", "action", "label", 999999.3);
  metrics.recordFloatingPointEvent("category", "action", "label", 999999.4, 'client id');
