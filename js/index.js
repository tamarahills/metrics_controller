/*
 * metrics.js
 *
 * Javascript implementation of CD Metrics library. The library
 * facilitates recording and sending metrics data to a server.
 * The library uses the Google Analytics Measurement Protocol.
 * The data on the server is exported to Google's BigQuery
 * which allows it to be accessed by data visualization tools
 * such as redash and Periscope.
 *
 */
const https = require("https");
var nconf = require('nconf');

/*
 * Constructor - Metrics(clientId, options)
 *
 *  clientId: Identifier that uniquely identifies the client.
 *            All metric data will be associated with this client id.
 *  options:  An object containing information about the client. All
 *            fields are optional.
 *   locale:             Locale or user language
 *   os:                 Operating system of the device
 *   os_version:         Version of the OS
 *   device:             Device name
 *   app_name:           Application name
 *   app_version:        Application version
 *   app_update_channel: Application update channel (e.g, nightly)
 *   app_platform:       Application platform
 *   arch:               Platform/device architecture
 */
function Metrics(clientId, options) {
    this.clientId = clientId;
    this.locale = options.locale || '';
    this.os = options.os || '';
    this.osVersion = options.osVersion || '';
    this.device = options.device || '';
    this.appName = options.appName || '';
    this.appVersion = options.appVersion || '';
    this.appUpdateChannel = options.app_update_channel || '';
    this.appPlatform = options.appPlatform || '';
    this.arch = options.arch || '';
    this.logger = options.logger;

    // Use nconf to get the configuration for different APIs we are using.
    var configFile = __dirname + '/config.json';
    this.log('config file: ' + configFile);

    nconf.argv()
       .env()
       .file({ file: configFile });
    this.analyticsProperty = nconf.get('analytics');
    this.log('this.analyticsProperty ' + this.analyticsProperty);
}

Metrics.prototype = {
    constructor: Metrics,

    /*
     * recordEvent - record an event (send data to the server)
     */
    recordEvent: function(event_category, // For example, 'eng', or 'user'
                          event_action,   // Action that triggered event (e.g., 'open-app')
                          event_label,    // Metric label (e.g., 'memory')
                          event_value,    // Value of metric (numeric)
                          clientId) {     // Client id (optional)
        var clientId = clientId || this.clientId;
        var self = this;

        var event_string = formatEventString();
        this.log("METRICS - event string:" + event_string);
        var post_options = {
            host: 'www.google-analytics.com',
            port: '443',
            path: '/batch',
            method: 'POST',
            headers: {
                'Content-Type': 'application/x-www-form-urlencoded',
                'Content-Length': event_string.length
            }
        };

        var post_req = https.request(post_options, function(res) {
            self.log("METRICS - request returned: " + res.statusCode);
            self.log('METRICS - event recorded: ' + event_category + ', '
                                                  + event_action +   ', '
                                                  + event_label +    ', '
                                                  + event_value);
        });

        // post the data
        this.log("METRICS - Sending request...");
        post_req.write(event_string);
        post_req.end();

        function formatEventString() {
            encodeURIComponent(event_category);
            encodeURIComponent(event_action);
            encodeURIComponent(event_label);
            encodeURIComponent(event_value);
            encodeURIComponent(clientId);

            encodeURIComponent(self.locale);
            encodeURIComponent(self.os);
            encodeURIComponent(self.osVersion);
            encodeURIComponent(self.device);
            encodeURIComponent(self.appName);
            encodeURIComponent(self.appVersion);
            encodeURIComponent(self.appUpdate_channel);
            encodeURIComponent(self.appPlatform);
            encodeURIComponent(self.arch);

            var event_string = ('v=1&t=event&tid=' + self.analyticsProperty +
                                '&cid=' + self.clientId +
                                '&ec=' + event_category +
                                '&ea=' + event_action +
                                '&el=' + event_label +
                                '&ev=' + event_value +
                                '&an=' + self.appName +
                                '&av=' + self.appVersion +
                                '&ul=' + self.locale +
                                '&cd1=' + self.os +
                                '&cd2=' + self.osVersion +
                                '&cd3=' + self.device +
                                '&cd4=' + self.arch +
                                '&cd5=' + self.appPlatform +
                                '&cd6=' + clientId + // Also store client id in cd6 because
                                                     // cid value is mangled by GA
                                '&cd7=' + getFormattedTime());

            return event_string;
        }

        function getFormattedTime() {
            var date = new Date();

            var month = date.getUTCMonth() + 1;
            var day = date.getUTCDate();
            var hour = date.getUTCHours();
            var min = date.getUTCMinutes();
            var sec = date.getUTCSeconds();

            month = (month < 10 ? "0" : "") + month;
            day = (day < 10 ? "0" : "") + day;
            hour = (hour < 10 ? "0" : "") + hour;
            min = (min < 10 ? "0" : "") + min;
            sec = (sec < 10 ? "0" : "") + sec;

            var str = date.getUTCFullYear() + "-" + month + "-" +  day + " " +  hour + ":" + min + ":" + sec;

            return str;
        }
    },

    recordEventAsync: function(event_category, // For example, 'eng', or 'user'
                               event_action,   // Action that triggered event (e.g., 'open-app')
                               event_label,    // Metric label (e.g., 'memory')
                               event_value,    // Value of metric (numeric)
                               clientId) {     // Client id (optional)
        var self = this;

        setTimeout(function() {
            self.recordEvent(event_category,
                             event_action,
                             event_label,
                             event_value,
                             clientId);
        }, 50);
    },

    recordFloatingPointEvent: function(event_category, // For example, 'eng', or 'user'
                                       event_action,   // Action that triggered event (e.g., 'open-app')
                                       event_label,    // Metric label (e.g., 'memory')
                                       event_value,    // Value of metric (numeric)
                                       clientId) {     // Client id (optional)
        var clientId = clientId || this.clientId;
        var self = this;

        var event_string = formatEventString();
        this.log("METRICS - event string:" + event_string);
        var post_options = {
            host: 'www.google-analytics.com',
            port: '443',
            path: '/batch',
            method: 'POST',
            headers: {
                'Content-Type': 'application/x-www-form-urlencoded',
                'Content-Length': event_string.length
            }
        };

        var post_req = https.request(post_options, function(res) {
            self.log("METRICS - request returned: " + res.statusCode);
            self.log('METRICS - event recorded: ' + event_category + ', '
                                                  + event_action +   ', '
                                                  + event_label +    ', '
                                                  + event_value);
        });

        // post the data
        this.log("METRICS - Sending request...");
        post_req.write(event_string);
        post_req.end();

        function formatEventString() {
            encodeURIComponent(event_category);
            encodeURIComponent(event_action);
            encodeURIComponent(event_label);
            encodeURIComponent(event_value);
            encodeURIComponent(clientId);

            encodeURIComponent(self.locale);
            encodeURIComponent(self.os);
            encodeURIComponent(self.osVersion);
            encodeURIComponent(self.device);
            encodeURIComponent(self.appName);
            encodeURIComponent(self.appVersion);
            encodeURIComponent(self.appUpdate_channel);
            encodeURIComponent(self.appPlatform);
            encodeURIComponent(self.arch);

            var event_string = ('v=1&t=event&tid=' + self.analyticsProperty +
                                '&cid=' + self.clientId +
                                '&ec=' + event_category +
                                '&ea=' + event_action +
                                '&el=' + event_label +
                                '&ev=' + 1 +
                                '&an=' + self.appName +
                                '&av=' + self.appVersion +
                                '&ul=' + self.locale +
                                '&cd1=' + self.os +
                                '&cd2=' + self.osVersion +
                                '&cd3=' + self.device +
                                '&cd4=' + self.arch +
                                '&cd5=' + self.appPlatform +
                                '&cd6=' + clientId + // Also store client id in cd6 because
                                                     // cid value is mangled by GA
                                '&cd7=' + getFormattedTime()) +
                                '&cd8=' + event_value ;

            return event_string;
        }

        function getFormattedTime() {
            var date = new Date();

            var month = date.getUTCMonth() + 1;
            var day = date.getUTCDate();
            var hour = date.getUTCHours();
            var min = date.getUTCMinutes();
            var sec = date.getUTCSeconds();

            month = (month < 10 ? "0" : "") + month;
            day = (day < 10 ? "0" : "") + day;
            hour = (hour < 10 ? "0" : "") + hour;
            min = (min < 10 ? "0" : "") + min;
            sec = (sec < 10 ? "0" : "") + sec;

            var str = date.getUTCFullYear() + "-" + month + "-" +  day + " " +  hour + ":" + min + ":" + sec;

            return str;
        }
    },

    recordFloatingPointEventAsync: function(event_category, // For example, 'eng', or 'user'
                                            event_action,   // Action that triggered event (e.g., 'open-app')
                                            event_label,    // Metric label (e.g., 'memory')
                                            event_value,    // Value of metric (numeric)
                                            clientId) {     // Client id (optional)
        var self = this;

        setTimeout(function() {
            self.recordFloatingPointEvent(event_category,
                                          event_action,
                                          event_label,
                                          event_value,
                                          clientId);
        }, 50);
    },
    log: function(msg) {
        if (this.logger) {
            this.logger(msg);
        }
        else {
            console.log(msg);
        }
    }
};

module.exports = Metrics;
