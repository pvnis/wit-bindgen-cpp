package wit.exports.test.records;

import wit.worlds.Records.Tuple1;
import wit.worlds.Records.Tuple2;
import wit.worlds.Records.Tuple3;

public class TestImpl {
    public static Tuple2<Byte, Short> multipleResults() {
        return new Tuple2<>((byte) 100, (short) 200);
    }

    public static Tuple2<Integer, Byte> swapTuple(Tuple2<Byte, Integer> tuple) {
        return new Tuple2<>(tuple.f1, tuple.f0);
    }

    public static Test.F1 roundtripFlags1(Test.F1 a) {
        return a;
    }

    public static Test.F2 roundtripFlags2(Test.F2 a) {
        return a;
    }

    public static Tuple3<Test.Flag8, Test.Flag16, Test.Flag32> roundtripFlags3
        (Test.Flag8 a, Test.Flag16 b, Test.Flag32 c)
    {
        return new Tuple3<>(a, b, c);
    }

    public static Test.R1 roundtripRecord1(Test.R1 a) {
        return a;
    }

    public static Tuple1<Byte> tuple1(Tuple1<Byte> a) {
        return a;
    }
}
