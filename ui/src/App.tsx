import type { Component } from 'solid-js';
import { Navbar } from './components/navbar';

const App: Component = () => {
  return (
    <div>
      <Navbar />
      <div>
        Lorem ipsum
      </div>
    </div>
  );
};

export default App;
