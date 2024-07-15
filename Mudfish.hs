import Network.Socket
import Network.HTTP.Proxy
import Network.HTTP.Client
import Network.HTTP.Conduit
import Text.HTML.Script

fetch :: Request -> IO Data

sanitizer :: Request -> IO (Either Response Request)
sanitizer req = do
  case fetch req of
    HTML html -> mapScripts sanitize $ inlineScripts html
    ECMAScript script -> ECMAScript $ sanitize $ script
    other -> other

runHTTPProxy :: Settings -> IO ()
runHTTPProxy = do
  port <- socketPort sock
  mgr <- newManager defaultManagerSettings
  Warp.runSettingsSocket (warpSettings set) sock
    $ httpProxyApp set mgr

main :: IO ()
main = do
  port <- lookupEnv "PORT"
  runHTTPProxy $ defaultProxySettings {
    proxyPort = fromMaybe 8080 $ port >>= readMaybe,
    proxyHttpRequestModifier = sanitizer,
    proxyLogger = putStrLn
    }
