//
// LiteX's watchdog core (litex.soc.cores.watchdog) as Zephyr's
// litex,watchdog driver programs it: control (feed bit 0, enable bit 8,
// reset bit 16), cycles, remaining, events. The count runs on the
// emulation's virtual clock; when it reaches zero the event is raised and,
// with the reset bit, the machine is reset. Feeds and expirations are
// counted for the bench.
//
using Antmicro.Renode.Core;
using Antmicro.Renode.Peripherals.Bus;
using Antmicro.Renode.Peripherals.Timers;
using Antmicro.Renode.Time;

namespace Antmicro.Renode.Peripherals.ArcLang
{
    public class LiteXWatchdog : IDoubleWordPeripheral, IWordPeripheral, IBytePeripheral, IKnownSize
    {
        public LiteXWatchdog(IMachine machine, ulong frequency = 100000000)
        {
            this.machine = machine;
            IRQ = new GPIO();
            timer = new LimitTimer(machine.ClockSource, frequency, this, "watchdog", ulong.MaxValue, Direction.Descending, false, WorkMode.OneShot, true);
            timer.LimitReached += Expire;
        }

        public long Size => 0x100;

        public GPIO IRQ { get; }

        public int Feeds { get; private set; }

        public int Expirations { get; private set; }

        public void Reset()
        {
            enabled = false;
            resetOnExpiry = false;
            pending = false;
            eventEnable = false;
            cycles = 0;
            timer.Enabled = false;
            IRQ.Unset();
        }

        public uint ReadDoubleWord(long offset)
        {
            switch(offset)
            {
            case 0x0: return enabled ? 1u : 0u;
            case 0x4: return (uint)cycles;
            case 0x8: return enabled ? (uint)timer.Value : 0u;
            case 0xC: return pending ? 1u : 0u;
            case 0x10: return pending ? 1u : 0u;
            case 0x14: return eventEnable ? 1u : 0u;
            }
            return 0;
        }

        public void WriteDoubleWord(long offset, uint value)
        {
            switch(offset)
            {
            case 0x0:
                enabled = (value & 0x100) != 0;
                resetOnExpiry = (value & 0x10000) != 0;
                if((value & 0x1) != 0 || !enabled)
                {
                    Restart();
                }
                break;
            case 0x4:
                cycles = value;
                break;
            case 0x10:
                if((value & 1) != 0)
                {
                    pending = false;
                    Update();
                }
                break;
            case 0x14:
                eventEnable = (value & 1) != 0;
                Update();
                break;
            }
        }

        // Zephyr feeds with an 8-bit write of the feed bit, and disables with a 16-bit write.
        public byte ReadByte(long offset)
        {
            return (byte)ReadDoubleWord(offset & ~3L);
        }

        public void WriteByte(long offset, byte value)
        {
            if(offset == 0x0)
            {
                if((value & 0x1) != 0)
                {
                    Feeds++;
                    Restart();
                }
                return;
            }
            WriteDoubleWord(offset, value);
        }

        public ushort ReadWord(long offset)
        {
            return (ushort)ReadDoubleWord(offset & ~3L);
        }

        public void WriteWord(long offset, ushort value)
        {
            if(offset == 0x0)
            {
                if((value & 0x100) != 0)
                {
                    enabled = false;
                    timer.Enabled = false;
                }
                return;
            }
            WriteDoubleWord(offset, value);
        }

        private void Restart()
        {
            timer.Enabled = false;
            if(enabled && cycles > 0)
            {
                timer.Limit = cycles;
                timer.Value = cycles;
                timer.Enabled = true;
            }
        }

        private void Expire()
        {
            Expirations++;
            pending = true;
            Update();
            if(resetOnExpiry)
            {
                machine.RequestReset();
            }
        }

        private void Update()
        {
            IRQ.Set(pending && eventEnable);
        }

        private readonly IMachine machine;
        private readonly LimitTimer timer;
        private bool enabled;
        private bool resetOnExpiry;
        private bool pending;
        private bool eventEnable;
        private ulong cycles;
    }
}
