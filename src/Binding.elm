port module Binding exposing (..)

import Test


port out : Int -> Cmd msg


main : Program Int () Never
main =
    Platform.worker
        { init = \x -> ( (), Test.add5 x |> out )
        , subscriptions = always Sub.none
        , update = \_ m -> ( m, Cmd.none )
        }
