import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Platform;

import java.io.File;
import java.io.FileOutputStream;
import java.io.PrintStream;

public class JavaMetrics {

    public interface RustLibrary extends Library {

        RustLibrary INSTANCE = (RustLibrary)
            Native.loadLibrary("metrics_controller",
                               RustLibrary.class);

        void init_metrics(String app_name,
                          String app_version,
                          String app_update_channel,
                          String app_build_id,
                          String app_platform,
                          String locale,
                          String device,
                          String arch,
                          String os,
                          String os_version);
        int record_event(String category, String action,
                         String label, int value);
        int record_floating_point_event(String category, String action,
                                        String label, double value);
    }

    public static void main(String[] args) throws InterruptedException {
      RustLibrary.INSTANCE.init_metrics("myapp",
                                        "1.0",
                                        "default",
                                        "20160303",
                                        "c",
                                        "en-us",
                                        "pi",
                                        "LAMP",
                                        "linux",
                                        "redhat");
      for (int i = 0; i < 5; i ++) {
          RustLibrary.INSTANCE.record_event("test", "click", "order", i);
          RustLibrary.INSTANCE.record_floating_point_event("test", "click", "order", i * .1);
      }
      Thread.sleep(30000);
    }
}
