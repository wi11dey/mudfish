module Language.JavaScript.Taint

import Language.JavaScript.Parser

sanitize :: Text -> Text
sanitize script =
  renderToText
  $ sanitizeAST
  $ readJs
  $ unpack script

sanitizeAST :: JSAST -> JSAST
sanitizeAST ast =

wrap :: JSExpression -> JSExpression
wrap exp =
  
