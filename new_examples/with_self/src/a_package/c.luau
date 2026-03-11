return require("@self/a") + require("@self").test2 -- test2 = 2, @self/a => 1 but prints 1,2,3,
