var Metrics = require('../js/index.js');

  var logger = function() {
    var args = Array.from(arguments);
    process.stdout.write(args.join(' ') + '\n');
  };

  var clientId = 222219191919;
  var options = {
      locale: 'locale',
      os: 'os',
      os_version: 'os_version',
      device: 'device',
      app_name: 'app_name',
      app_version: 'app_version',
      app_update_channel: 'app_update_channel',
      app_build_id: 'app_build_id',
      app_platform: 'app_platform',
      arch: 'arch',
      logger: logger
  };

  process.stdout.write("Instantiating Metrics object\n");
  var metrics = new Metrics(clientId, options);

  process.stdout.write("Recording events...\n");
  metrics.recordEventAsync("category", "action", "label", 987654321);
  metrics.recordEvent("category", "action", "label", 987654322);
  metrics.recordFloatingPointEvent("category", "action", "label", 999999.9);
