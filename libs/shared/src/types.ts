export interface Task {
  id: string;
  title: string;
  description: string | null;
  completed: boolean;
}

export interface CreateTask {
  title: string;
  description: string;
  completed: boolean;
}

export interface UpdateTask {
  title?: string;
  description?: string;
  completed?: boolean;
}