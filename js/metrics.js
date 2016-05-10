var Metrics = (function() {

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
    
    Metrics.prototype.recordEvent = function(event_category,
                                             event_action,
                                             event_label,
                                             event_value) {
    
        var self = this;

        var xhr = new XMLHttpRequest();
    
        xhr.open('POST', "https://www.google-analytics.com/batch");
    
        xhr.timeout = 3000;
    
        //xhr.setRequestHeader('Content-type', 'application/json');
        xhr.responseType = 'text';
        var event_string = formatEventString();
    
        console.log('METRICS - event string:', event_string);
     
        xhr.send(event_string);
    
        function onload() {
            console.log('METRICS - event recorded:', event_category, ',',
                                                     event_action, ',',
                                                     event_label, ',',
                                                     event_value);
        }
    
        function onerror(e) {
            console.log('METRICS - error recording event :', e.type);
        }
/*    
        function generateCid() {
            return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
                var r = Math.random()*16|0, v = c == 'x' ? r : (r&0x3|0x8);
                return v.toString(16);
            });    
        }
 */ 
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
    
        xhr.onload = onload;
        xhr.onerror = onerror;
        xhr.onabort = onerror;
        xhr.ontimeout = onerror;
      };
    
      return Metrics;

}());

module.exports = Metrics;
