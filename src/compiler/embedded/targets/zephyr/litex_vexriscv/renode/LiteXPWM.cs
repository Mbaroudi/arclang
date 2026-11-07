//
// LiteX's PWM core (litex.soc.cores.pwm, CSR data width 32): enable,
// width and period, as Zephyr's litex,pwm driver programs them. Renode has
// no model of it (it tags the range). The bench reads the duty the driver
// programmed (Permille): the waveform itself is not generated.
//
using Antmicro.Renode.Core;
using Antmicro.Renode.Peripherals.Bus;

namespace Antmicro.Renode.Peripherals.ArcLang
{
    public class LiteXPWM : IDoubleWordPeripheral, IBytePeripheral, IKnownSize
    {
        public LiteXPWM(IMachine machine)
        {
        }

        public long Size => 0x100;

        public void Reset()
        {
            enable = 0;
            width = 0;
            period = 0;
        }

        // The duty in per mille while enabled, else 0.
        public int Permille
        {
            get
            {
                if(enable == 0 || period == 0)
                {
                    return 0;
                }
                return (int)(((ulong)width * 1000UL) / period);
            }
        }

        public uint ReadDoubleWord(long offset)
        {
            switch(offset)
            {
            case 0x0: return enable;
            case 0x4: return width;
            case 0x14: return period;
            }
            return 0;
        }

        public void WriteDoubleWord(long offset, uint value)
        {
            switch(offset)
            {
            case 0x0: enable = value & 1; break;
            case 0x4: width = value; break;
            case 0x14: period = value; break;
            }
        }

        public byte ReadByte(long offset)
        {
            return (byte)ReadDoubleWord(offset);
        }

        public void WriteByte(long offset, byte value)
        {
            if(offset == 0x0)
            {
                enable = (uint)(value & 1);
            }
        }

        private uint enable;
        private uint width;
        private uint period;
    }
}
