package expo.modules.myrustmodule

import kotlinx.coroutines.*
import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition

class MyRustModule : Module() {
    companion object {
        init {
            System.loadLibrary("blockmesh_cli")
        }
    }

    external fun runLib(url: String, email: String, password: String): Int
    external fun stopLib(url: String): Int
    external fun getLibStatus(): Int
  // Each module class must implement the definition function. The definition consists of components
  // that describes the module's functionality and behavior.
  // See https://docs.expo.dev/modules/module-api for more details about available components.
  override fun definition() = ModuleDefinition {
    // Sets the name of the module that JavaScript code will use to refer to the module. Takes a string as an argument.
    // Can be inferred from module's class name, but it's recommended to set it explicitly for clarity.
    // The module will be accessible from `requireNativeModule('MyRustModule')` in JavaScript.
    Name("MyRustModule")

    // Sets constant properties on the module. Can take a dictionary or a closure that returns a dictionary.
    Constants(
      "PI" to Math.PI
    )

    // Defines event names that the module can send to JavaScript.
    Events("onChange")

    // Defines a JavaScript synchronous function that runs the native code on the JavaScript thread.
    Function("hello") {
      "Hello world! 👋"
    }

    AsyncFunction("run_lib") { url: String, email: String, password: String ->
        runLib(url, email, password)
    }

    AsyncFunction("stop_lib") {url: string ->
        stopLib(url)
    }

    Function("get_lib_status") {
        return getLibStatus()
    }

    // Defines a JavaScript function that always returns a Promise and whose native code
    // is by default dispatched on the different thread than the JavaScript runtime runs on.
    AsyncFunction("setValueAsync") { value: String ->
      // Send an event to JavaScript.
      sendEvent("onChange", mapOf(
        "value" to value
      ))
    }

    // Enables the module to be used as a native view. Definition components that are accepted as part of
    // the view definition: Prop, Events.
    View(MyRustModuleView::class) {
      // Defines a setter for the `name` prop.
      Prop("name") { view: MyRustModuleView, prop: String ->
        println(prop)
      }
    }
  }
}
