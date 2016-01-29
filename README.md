主要是模仿Go的text/template/parse里面的代码，lex部分已经完成，通过了和Go一样的测试（部分字符Rust不支持所以有点不一样）。

parse部分浪费了我很久的时间，主要是ListNode这里，一个Vec<Box<Node>>, 因为trait object不支持反向得出concrete type，我浪费了非常多时间在这一块，寻找解决办法。

弄了几个星期，突然心情大冷，决定先搁置一段时间再来重写这部分代码
