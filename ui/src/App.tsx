import type { Component } from 'solid-js';
import { Navbar } from '#/components/navbar';
import { Sidebar } from '#/components/sidebar';

const App: Component = () => {
  return (
    <div class="h-screen flex flex-col">
      <Navbar />
      <div class="flex h-full">
        <Sidebar />
        <div class="overflow-y-auto">
          <div class="p-4">
            Lorem ipsum
          </div>
        </div>
      </div>
    </div>
  );
};

export default App;
