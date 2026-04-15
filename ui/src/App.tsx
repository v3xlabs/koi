import type { Component } from 'solid-js';
import { Navbar } from '#/components/navbar';
import { Sidebar } from '#/components/sidebar';
import { FaSolidArrowRight, FaSolidRefresh } from 'solid-icons/fa';

const App: Component = () => {
  return (
    <div class="h-screen flex flex-col">
      <Navbar />
      <div class="flex h-full">
        <Sidebar />
        <div class="overflow-y-auto w-full">
          <div class="p-4 grid grid-cols-5 w-full gap-4">
            <div class="bg-surface p-4 col-span-3 rounded-md space-y-4">
              <div class="flex justify-between items-center">
                <div class="text-sm font-bold text-muted">
                  Total balance
                </div>
                <div class="text-muted text-sm flex items-center gap-2">
                  <span>
                    Updated just now
                  </span>
                  <FaSolidRefresh class="w-3.5 h-3.5 text-primary-foreground" />
                </div>
              </div>
              <div class="flex justify-between items-center">
                <div class="text-4xl font-bold tabular-nums">
                  <span class="text-foreground">
                    $100,100
                  </span>
                  <span class="text-muted">.00</span>
                </div>
                <div class="flex gap-2">
                  <button
                    class="bg-primary hover:bg-primary-hover text-primary-foreground w-full rounded-md py-2 px-4 flex items-center gap-2 cursor-pointer justify-center text-sm font-bold"
                  >
                    <FaSolidArrowRight class="-rotate-45" />
                    Send
                  </button>
                  <button
                    class="bg-secondary hover:bg-secondary-hover text-primary-foreground w-full rounded-md py-2 px-4 flex items-center gap-2 cursor-pointer justify-center text-sm font-bold"
                  >
                    <FaSolidArrowRight class="-rotate-225" />
                    Receive
                  </button>
                </div>
              </div>
            </div>
            <div class="bg-surface p-4 col-span-2 rounded-md">
              <div>
                <div>
                  Pending transactions
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default App;
