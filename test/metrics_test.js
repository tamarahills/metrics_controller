var expect = chai.expect;

describe("Metrics", function() {
  describe("recordEvent", function() {
    it("should be received to the server", function(done) {
      this.timeout(400000);
      var event_category     = "test";
      var event_action       = "integration";
      var event_label        = 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
                                  var r = Math.random()*16|0, v = c == 'x' ? r : (r&0x3|0x8);
                                  return v;
                                });
      var event_value        = 999999;

      var opt = {
        locale : 'en-us',
        os : 'linux'
      };

      // Obtain the access token from the Query Explorer:  https://ga-dev-tools.appspot.com/query-explorer/.
      // 1. Run a Query (any query).  2.  Scroll down to the "API Query URI" box.  3.  Check the checkbox
      // "Include current access_token in the Query URI (will expire in ~60 minutes)."  4.  Copy the
      // access_token parameter and paste into the line below.
      var access_token = 'ya29.CjPvAiqOyUuO80MfKwY2sqaPOOJ6gk5TXa_YPacCwRRjWvvO8Fm8RtnOM52tmg-ajs1boF4';

      var metrics = new Metrics('1234', opt);
      metrics.recordEvent(event_category, event_action, event_label, event_value);

      var timer = setInterval(function() {
        var now = new Date();
        var date = stringDate = now.getFullYear() + '-' +
                                      ('0' + (now.getMonth() + 1)).slice(-2) + '-' +
                                      ('0' + now.getDate()).slice(-2);
        var report_url = 'https://www.googleapis.com/analytics/v3/data/ga?ids=ga%3A121095747' +
                         '&start-date=' + date +
                         '&end-date=' + date +
                         '&metrics=ga%3AeventValue&dimensions=ga%3AeventCategory%2Cga%3AeventAction' +
                         '%2Cga%3AeventLabel' +
                         '&filters=ga%3AeventLabel%3D%3D' + event_label +
                         '&access_token=' + access_token;
        console.log('Report url is: ' + report_url);

        var xhr = new XMLHttpRequest();
        xhr.open('GET', report_url);
        xhr.timeout = 3000;
        xhr.responseType = 'text';
        xhr.send(report_url);
        console.log('Sending request');

        function onload() {
          console.log('METRICS_TEST - request received:');
        }

        function onerror(e) {
           console.log('METRICS_TEST - error getting report :', e.type);
        }

        xhr.onload = onload;
        xhr.onerror = onerror;
        xhr.onabort = onerror;
        xhr.ontimeout = onerror;
        xhr.onreadystatechange = function() {
          if (xhr.readyState == 4 && xhr.status == 200) {
            console.log('METRICS_TEST' + xhr.responseText);
            obj = JSON.parse(xhr.responseText);
            var obj_body = obj['totalsForAllResults']['ga:eventValue'];
            if (obj_body == event_value) {
              clearTimeout(timer);
              done();
            }
            console.log('METRICS_TEST: ' + obj_body);
          }
        };
      }, 10000);
    });
  });
});
