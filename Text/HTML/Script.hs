module Text.HTML.Script (inlineScripts, mapScripts) where

import Text.XML
import Data.Text

isScript element = toLower $ nameLocalName $ elementName element == "script"

inlineScripts :: Document -> IO Document
inlineScripts doc = do
  root <- inlineElementScripts $ documentRoot doc
  return doc { documentRoot = root }
  where
    inlineElementScripts :: Element -> IO Element
    inlineElementScripts element
      | isScript element =
      | otherwise = return element

mapScripts :: (Text -> Text) -> Document -> Document
mapScripts f doc = doc {
  documentRoot = mapElementScripts $ documentRoot doc
  }
  where
    mapNodeScripts :: Node -> Node
    mapNodeScripts (NodeElement element) = NodeElement $ mapElementScripts element
    mapNodeScripts node = node

    mapElementScripts :: Element -> Element
    mapElementScripts element = element {
      elementAttributes = mapWithKey mapAttribute $ elementAttributes element
      elementNodes = if not $ isScript element then elementNodes element else
          case elementNodes element of
            [NodeContent script] -> [NodeContent $ f script]
            _ -> [] -- non-inline or invalid script
      }

    -- Also map over the inline scripts present in "on*" attributes:
    mapAttribute :: Name -> Text -> Text
    mapAttribute key value
      | "on" `isPrefixOf` (toLower $ nameLocalName key) = f value
      | otherwise = value
