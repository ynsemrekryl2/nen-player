// NEN-011 — Kotlin/JVM tarafında Rust `spike-cue-transfer`'ın unit testlerinin
// (src/lib.rs `mod tests`) altkümesi: iki yaklaşımın (A tam liste, B pencere)
// aynı veriyi döndürdüğünü doğruluyor. Swift'in ayrı bir test target'ı yok
// (NEN-008 doğruluğunu Rust tarafında zaten kanıtlıyor); bu test I5'in
// ("semantik sonuçlar Swift ve Kotlin arasında aynı") Kotlin ayağı — iki
// binding de aynı Rust crate'ini çağırdığı için asıl kanıt Rust tarafında,
// burada yalnız Kotlin binding'in doğru marshalling yaptığı doğrulanıyor.

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue
import uniffi.spike_cue_transfer.CueHandle
import uniffi.spike_cue_transfer.allCues

class CueTransferParityTest {
    private val n = 1_000u

    @Test
    fun windowMatchesTheSameSliceOfTheFullList() {
        val full = allCues(n)
        val handle = CueHandle(n)

        assertEquals(n, handle.count())
        assertEquals(full.subList(0, 40), handle.cues(0u, 40u))
        assertEquals(full.subList(500, 540), handle.cues(500u, 40u))
        assertEquals(full.subList((n - 10u).toInt(), full.size), handle.cues(n - 10u, 40u))
        handle.destroy()
    }

    @Test
    fun windowPastTheEndIsEmptyNotAnError() {
        val handle = CueHandle(n)
        assertTrue(handle.cues(n, 40u).isEmpty())
        assertTrue(handle.cues(n + 5_000u, 40u).isEmpty())
        assertTrue(handle.cues(0u, 0u).isEmpty())
        handle.destroy()
    }

    @Test
    fun activeCueIsNoneInTheGapsAndOutsideTheDocument() {
        val handle = CueHandle(n)
        val first = allCues(1u)[0]
        val last = allCues(n).last()

        assertNull(handle.activeCue(first.endMs))
        assertNull(handle.activeCue(first.endMs + 100uL))
        assertNull(handle.activeCue(last.endMs))
        assertNull(handle.activeCue(last.endMs + 60_000uL))
        handle.destroy()
    }
}
