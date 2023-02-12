
#include<std/io>
#include<std/assert>

mod intern {
    using intern
    using io
    using assert
    
    fn test -> {
        "Testing string interning... " print

        "\"bf121c3c271c\" = \"bf121c3c271c\"" { "bf121c3c271c" "bf121c3c271c" = } assert
        "\"bf121c3c271c\" = \"1c3c271c\" - 4" { "bf121c3c271c" "1c3c271c" 4 - = } assert
        "\"bf121c3c271c\" = \"deadbf121c3c271c\" + 4" { "bf121c3c271c" "deadbf121c3c271c" 4 + = } assert
    }
}


fn test_intern -> {
    intern::test
}