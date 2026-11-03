//
// The ADC of ArcLang's LiteX target: one 32-bit read-only register per
// channel holding the last raw conversion (0 to 65535), as a LiteX
// CSRStatus bank fed by a converter interface would hold it. LiteX has no
// ADC core: this register bank is this target's own definition. The
// bench sets the raw values (SetRaw); the CPU reads them through Zephyr's
// ADC API (driver arclang,litex-adc).
//
using Antmicro.Renode.Core;
using Antmicro.Renode.Peripherals.Bus;

namespace Antmicro.Renode.Peripherals.ArcLang
{
    public class RegisterADC : IDoubleWordPeripheral, IKnownSize
    {
        public RegisterADC(IMachine machine, int channels = 8)
        {
            raw = new uint[channels];
        }

        public long Size => 0x100;

        public void Reset()
        {
            for(var k = 0; k < raw.Length; k++)
            {
                raw[k] = 0;
            }
        }

        public void SetRaw(int channel, uint value)
        {
            raw[channel] = value & 0xFFFF;
        }

        public uint ReadDoubleWord(long offset)
        {
            var k = (int)(offset / 4);
            return k < raw.Length ? raw[k] : 0u;
        }

        public void WriteDoubleWord(long offset, uint value)
        {
        }

        private readonly uint[] raw;
    }
}
