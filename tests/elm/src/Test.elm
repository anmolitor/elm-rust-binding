module Test exposing (..)


add5 : Int -> Int
add5 =
    (+) 5


prependTest : String -> String
prependTest =
    (++) "test"


type alias StructIn =
    { a : Maybe Int, b : List Bool }


type alias StructOut =
    { c : List Int, d : Maybe Bool }


someStructMapper : List StructIn -> List StructOut
someStructMapper =
    let
        mapOne { a, b } =
            { c =
                case a of
                    Just a_ ->
                        [ a_ ]

                    Nothing ->
                        []
            , d = List.head b
            }
    in
    List.map mapOne
