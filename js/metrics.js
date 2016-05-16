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
(function(exports) {
  'use strict';

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
  *   app_build_id:       Application build Id
  *   app_platform:       Application platform
  *   arch:               Platform/device architecture
  */
  function Metrics(clientId, options) {
      this.clientId = clientId;
      this.locale = options.locale || '';
      this.os = options.os || '';
      this.os_version = options.os_version || '';
      this.device = options.device || '';
      this.app_name = options.app_name || '';
      this.app_version = options.app_version || '';
      this.app_update_channel = options.app_update_channel || '';
      this.app_build_id = options.app_build_id || '';
      this.app_platform = options.app_platform || '';
      this.arch = options.arch || '';
  }
  exports.Metrics = Metrics;

 /*
  * recordEvent - record an event (send data to the server)
  */
  Metrics.prototype.recordEvent = function(event_category, // For example, 'eng', or 'user'
                                           event_action,   // Action that triggered event (e.g., 'open-app')
                                           event_label,    // Metric label (e.g., 'memory')
                                           event_value) {  // Value of metric (numeric)
      var self = this;
      var post_url = 'https://www.google-analytics.com/batch';
      var event_string = formatEventString();
      console.log('METRICS - event string:', event_string);

      // The fetch interface is used by newer browsers while the XHR is used by
      // older versions of browsers including Safari which currently does not
      // support fetch interface.
      if(this.fetch) {
        var init = { method: 'POST', body: event_string};
        fetch(post_url, init)
        .then(function(response) {
          if (response.ok) {
            console.log('METRICS - Success');
          } else {
            console.log('METRICS - Error: ' + response.status);
          }
        });
      } else {
        var xhr = new XMLHttpRequest();
        xhr.open('POST', post_url);
        xhr.timeout = 3000;

        xhr.responseType = 'text';

        xhr.send(event_string);

        xhr.onload = onload;
        xhr.onerror = onerror;
        xhr.onabort = onerror;
        xhr.ontimeout = onerror;
      }

      function onload() {
          console.log('METRICS - event recorded:', event_category, ',',
                                                   event_action, ',',
                                                   event_label, ',',
                                                   event_value);
      }

      function onerror(e) {
          console.log('METRICS - error recording event :', e.type);
      }

      function formatEventString() {
          encodeURIComponent(event_category);
          encodeURIComponent(event_action);
          encodeURIComponent(event_label);
          encodeURIComponent(event_value);

          encodeURIComponent(self.locale);
          encodeURIComponent(self.os);
          encodeURIComponent(self.os_version);
          encodeURIComponent(self.device);
          encodeURIComponent(self.app_name);
          encodeURIComponent(self.app_version);
          encodeURIComponent(self.app_update_channel);
          encodeURIComponent(self.app_build_id);
          encodeURIComponent(self.app_platform);
          encodeURIComponent(self.arch);

          var event_string = ('v=1&t=event&tid=UA-77033033-1&cid=' + self.clientId +
                              '&ec=' + event_category +
                              '&ea=' + event_action +
                              '&el=' + event_label +
                              '&ev=' + event_value +
                              '&an=' + self.app_name +
                              '&av=' + self.app_version +
                              '&ul=' + self.locale +
                              '&cd1=' + self.os +
                              '&cd2=' + self.os_version +
                              '&cd3=' + self.device +
                              '&cd4=' + self.arch +
                              '&cd5=' + self.app_platform +
                              '&cd6=' + self.app_build_id);

          return event_string;
      }
  };
})(this);
