"use client";

import { useState } from "react";
import {
  DndContext,
  PointerSensor,
  useSensor,
  useSensors,
  closestCenter,
  type DragEndEvent,
} from "@dnd-kit/core";
import {
  SortableContext,
  arrayMove,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { Clock, GripVertical, ListMusic, Play, Plus, Trash2, X } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { IconButton } from "@/components/ui/IconButton";
import { useStore } from "@/lib/store";
import { cn, formatTime } from "@/lib/utils";

export function PlaylistsView() {
  const playlists = useStore((s) => s.playlists);
  const navigate = useStore((s) => s.navigate);
  const createPlaylist = useStore((s) => s.createPlaylist);
  const deletePlaylist = useStore((s) => s.deletePlaylist);
  const [creating, setCreating] = useState(false);
  const [newName, setNewName] = useState("");

  const submit = async () => {
    const name = newName.trim();
    if (!name) return;
    const pl = await createPlaylist(name);
    setNewName("");
    setCreating(false);
    if (pl) navigate({ name: "playlist", id: pl.id });
  };

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center justify-between px-6 py-4 border-b border-border">
        <div>
          <h2 className="text-xl font-bold">Playlists</h2>
          <p className="text-sm text-text-muted mt-0.5">{playlists.length} playlists</p>
        </div>
        <Button variant="primary" size="sm" onClick={() => setCreating(true)}>
          <Plus size={16} /> New Playlist
        </Button>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        {creating && (
          <div className="mb-4 flex items-center gap-2 bg-bg-elevated border border-border rounded-lg p-3">
            <ListMusic size={18} className="text-accent" />
            <input
              autoFocus
              type="text"
              placeholder="Playlist name…"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") submit();
                if (e.key === "Escape") setCreating(false);
              }}
              className="flex-1 bg-transparent text-sm outline-none placeholder:text-text-muted"
            />
            <Button size="sm" variant="primary" onClick={submit}>
              Create
            </Button>
            <IconButton label="Cancel" onClick={() => setCreating(false)}>
              <X size={16} />
            </IconButton>
          </div>
        )}

        {playlists.length === 0 && !creating ? (
          <div className="flex h-full flex-col items-center justify-center text-center text-text-muted gap-3">
            <ListMusic size={48} className="opacity-40" />
            <p>No playlists yet.</p>
            <Button variant="secondary" size="sm" onClick={() => setCreating(true)}>
              <Plus size={16} /> Create your first playlist
            </Button>
          </div>
        ) : (
          <div className="grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-3">
            {playlists.map((pl) => (
              <div
                key={pl.id}
                className="group relative rounded-lg border border-border bg-bg-elevated hover:border-border-strong transition-colors cursor-pointer"
                onClick={() => navigate({ name: "playlist", id: pl.id })}
              >
                <div className="p-4">
                  <div className="flex items-start justify-between">
                    <div className="flex items-center justify-center w-10 h-10 rounded-md bg-accent/10 text-accent">
                      <ListMusic size={18} />
                    </div>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        deletePlaylist(pl.id);
                      }}
                      className="text-text-muted hover:text-danger opacity-0 group-hover:opacity-100 transition-opacity"
                      aria-label="Delete playlist"
                    >
                      <Trash2 size={15} />
                    </button>
                  </div>
                  <div className="mt-3 font-semibold text-sm truncate">{pl.name}</div>
                  <div className="text-xs text-text-muted mt-0.5">{pl.track_count} tracks</div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export function PlaylistDetailView({ playlistId }: { playlistId: string }) {
  const playlists = useStore((s) => s.playlists);
  const tracksById = useStore((s) => s.tracksById);
  const playlistTracks = useStore((s) => s.playlistTracks);
  const playTrack = useStore((s) => s.playTrack);
  const renamePlaylist = useStore((s) => s.renamePlaylist);
  const reorderPlaylistTracks = useStore((s) => s.reorderPlaylistTracks);
  const currentTrackId = useStore((s) => s.currentTrackId);
  const navigate = useStore((s) => s.navigate);

  const playlist = playlists.find((p) => p.id === playlistId);
  const [editingName, setEditingName] = useState(false);
  const [nameDraft, setNameDraft] = useState(playlist?.name ?? "");

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } }),
  );

  if (!playlist) {
    return (
      <div className="flex h-full flex-col items-center justify-center text-text-muted gap-3">
        <p>Playlist not found.</p>
        <Button variant="secondary" size="sm" onClick={() => navigate({ name: "playlists" })}>
          Back to playlists
        </Button>
      </div>
    );
  }

  const trackIds = playlistTracks[playlistId] ?? [];
  const tracks = trackIds.map((id) => tracksById[id]).filter(Boolean);

  const playAll = () => {
    if (tracks.length > 0) playTrack(tracks[0]!, tracks);
  };

  const commitRename = async () => {
    const name = nameDraft.trim();
    if (name && name !== playlist.name) await renamePlaylist(playlist.id, name);
    setEditingName(false);
  };

  const handleDragEnd = (e: DragEndEvent) => {
    const { active, over } = e;
    if (!over || active.id === over.id) return;
    const oldIndex = trackIds.indexOf(String(active.id));
    const newIndex = trackIds.indexOf(String(over.id));
    if (oldIndex < 0 || newIndex < 0) return;
    const reordered = arrayMove(trackIds, oldIndex, newIndex);
    void reorderPlaylistTracks(playlist.id, reordered);
  };

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-end gap-6 px-6 py-6 border-b border-border">
        <div className="flex items-center justify-center w-44 h-44 rounded-xl bg-gradient-to-br from-accent/30 to-accent/5 border border-border">
          <ListMusic size={56} className="text-accent" />
        </div>
        <div className="flex flex-col gap-2 pb-2 min-w-0 flex-1">
          <span className="text-xs uppercase tracking-wider text-text-muted">Playlist</span>
          {editingName ? (
            <input
              autoFocus
              type="text"
              value={nameDraft}
              onChange={(e) => setNameDraft(e.target.value)}
              onBlur={commitRename}
              onKeyDown={(e) => {
                if (e.key === "Enter") commitRename();
                if (e.key === "Escape") setEditingName(false);
              }}
              className="text-4xl font-bold bg-transparent border-b border-accent outline-none"
            />
          ) : (
            <button
              onClick={() => {
                setNameDraft(playlist.name);
                setEditingName(true);
              }}
              className="text-4xl font-bold text-left truncate hover:text-accent transition-colors"
            >
              {playlist.name}
            </button>
          )}
          <p className="text-text-secondary">{tracks.length} tracks</p>
          <div className="mt-3">
            <button
              onClick={playAll}
              disabled={tracks.length === 0}
              className="inline-flex items-center gap-2 bg-accent text-zinc-950 font-semibold rounded-full h-10 px-6 hover:bg-accent-hover transition-colors disabled:opacity-50"
            >
              <Play size={16} fill="currentColor" /> Play
            </button>
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        {tracks.length === 0 ? (
          <div className="flex h-full flex-col items-center justify-center text-center text-text-muted gap-2 py-20">
            <ListMusic size={40} className="opacity-40" />
            <p className="text-sm">This playlist is empty.</p>
            <p className="text-xs">Browse the library and add tracks to get started.</p>
          </div>
        ) : (
          <>
            <div className="flex items-center gap-4 px-6 text-xs uppercase tracking-wider text-text-muted border-b border-border h-10">
              <span className="w-6" />
              <span className="flex-1">Title</span>
              <span className="w-44">Artist</span>
              <span className="w-16 text-right">
                <Clock size={13} />
              </span>
            </div>
            <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={handleDragEnd}>
              <SortableContext items={tracks.map((t) => t!.id)} strategy={verticalListSortingStrategy}>
                {tracks.map((t, i) => (
                  <SortableRow
                    key={t!.id}
                    id={t!.id}
                    index={i}
                    title={t!.title}
                    artist={t!.artist}
                    duration={t!.duration_secs}
                    isCurrent={t!.id === currentTrackId}
                    onPlay={() => playTrack(t!, tracks)}
                  />
                ))}
              </SortableContext>
            </DndContext>
          </>
        )}
      </div>
    </div>
  );
}

function SortableRow({
  id,
  index,
  title,
  artist,
  duration,
  isCurrent,
  onPlay,
}: {
  id: string;
  index: number;
  title: string;
  artist: string;
  duration: number;
  isCurrent: boolean;
  onPlay: () => void;
}) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    zIndex: isDragging ? 50 : undefined,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={cn(
        "flex items-center gap-4 px-6 h-12 group hover:bg-bg-hover cursor-default",
        isDragging && "bg-bg-active border-t border-b border-border-strong shadow-xl",
      )}
      onDoubleClick={onPlay}
    >
      <span className="w-6 text-right text-sm text-text-muted group-hover:hidden">{index + 1}</span>
      <button
        {...attributes}
        {...listeners}
        className="w-6 hidden group-hover:flex justify-center items-center text-text-muted hover:text-text cursor-grab active:cursor-grabbing"
        aria-label="Drag to reorder"
      >
        <GripVertical size={14} />
      </button>
      <div className="flex-1 min-w-0">
        <span className={cn("truncate text-sm", isCurrent ? "text-accent" : "text-text")}>{title}</span>
      </div>
      <span className="w-44 truncate text-sm text-text-secondary">{artist}</span>
      <span className="w-16 text-right text-sm text-text-muted tabular-nums">{formatTime(duration)}</span>
    </div>
  );
}
