import { createSignal } from 'solid-js';
import type { Component } from 'solid-js';
import { RegularChat } from './components/RegularChat';
import { StreamingChat } from './components/StreamingChat';
import { Button } from './components/ui/button';
import './App.css';

const App: Component = () => {
  const [currentPage, setCurrentPage] = createSignal<'regular' | 'streaming'>('regular');

  return (
    <div class="flex min-h-screen flex-col">
      <nav class="border-b p-5 mb-5">
        <h1 class="text-4xl font-bold">AI Agent Chat</h1>
        <div class="flex gap-2.5 mt-2.5">
          <Button
            onClick={() => setCurrentPage('regular')}
            variant={currentPage() === 'regular' ? 'default' : 'secondary'}
          >
            Regular Chat
          </Button>
          <Button
            onClick={() => setCurrentPage('streaming')}
            variant={currentPage() === 'streaming' ? 'default' : 'secondary'}
          >
            Streaming Chat
          </Button>
        </div>
      </nav>

      <div class="px-5 flex-1">
        {currentPage() === 'regular' && <RegularChat />}
        {currentPage() === 'streaming' && <StreamingChat />}
      </div>
    </div>
  );
};

export default App;
