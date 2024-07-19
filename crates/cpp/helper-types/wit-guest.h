#include "wit-common.h"
#include <malloc.h>
#include <memory> // unique_ptr
#include <stdint.h>
#include <string>
#include <string_view>
#include <string.h> // memcpy

namespace wit {
/// A string in linear memory, freed unconditionally using free
///
/// A normal C++ string makes no guarantees about where the characters
/// are stored and how this is freed.
class string {
  uint8_t const *data_;
  size_t length;

public:
  string(string const &) = delete;
  string(string &&b) : data_(b.data_), length(b.length) { b.data_ = nullptr; }
  string &operator=(string const &) = delete;
  string &operator=(string &&b) {
    if (data_) {
      free(const_cast<uint8_t *>(data_));
    }
    data_ = b.data_;
    length = b.length;
    b.data_ = nullptr;
    return *this;
  }
  string(char const *d, size_t l) : data_((uint8_t const *)d), length(l) {}
  char const *data() const { return (char const *)data_; }
  size_t size() const { return length; }
  ~string() {
    if (data_) {
      free(const_cast<uint8_t *>(data_));
    }
  }
  // leak the memory
  void leak() { data_ = nullptr; }
  // typically called by post
  static void drop_raw(void *ptr) { free(ptr); }
  std::string_view get_view() const {
    return std::string_view((const char *)data_, length);
  }
  std::string to_string() const {
    return std::string((const char *)data_, length);
  }
  static string from_view(std::string_view v) {
    char* addr = (char*)malloc(v.size());
    memcpy(addr, v.data(), v.size());
    return string(addr, v.size());
  }
};

/// A vector in linear memory, freed unconditionally using free
///
/// You can't detach the data memory from a vector, nor create one
/// in a portable way from a buffer and lenght without copying.
template <class T> class vector {
  T *data_;
  size_t length;

public:
  vector(vector const &) = delete;
  vector(vector &&b) : data_(b.data_), length(b.length) { b.data_ = nullptr; }
  vector &operator=(vector const &) = delete;
  vector &operator=(vector &&b) {
    if (data_) {
      free(const_cast<uint8_t *>(data_));
    }
    data_ = b.data_;
    length = b.length;
    b.data_ = nullptr;
    return *this;
  }
  vector(T *d, size_t l) : data_(d), length(l) {}
  T const *data() const { return data_; }
  T *data() { return data_; }
  T &operator[](size_t n) { return data_[n]; }
  T const &operator[](size_t n) const { return data_[n]; }
  size_t size() const { return length; }
  ~vector() {
    if (data_) {
      free(data_);
    }
  }
  // leak the memory
  void leak() { data_ = nullptr; }
  // typically called by post
  static void drop_raw(void *ptr) { free(ptr); }
  wit::span<T> get_view() const { return wit::span<T>(data_, length); }
};

/// @brief  A Resource defined within the guest (guest side)
///
/// It registers with the host and should remain in a static location.
/// Typically referenced by the Owned type
///
/// Note that deregistering will cause the host to call Dtor which
/// in turn frees the object.
template <class R> class ResourceExportBase {
public:
  struct Deregister {
    void operator()(R *ptr) const {
      // probably always true because of unique_ptr wrapping, TODO: check
#ifdef WIT_SYMMETRIC
      if (ptr->handle != nullptr)
#else
      if (ptr->handle >= 0)
#endif
      {
        // we can't deallocate because the host calls Dtor
        R::ResourceDrop(ptr->handle);
      }
    }
  };
  typedef std::unique_ptr<R, Deregister> Owned;

#ifdef WIT_SYMMETRIC
  typedef uint8_t *handle_t;
  static constexpr handle_t invalid = nullptr;
#else
  typedef int32_t handle_t;
  static const handle_t invalid = -1;
#endif

  handle_t handle;

  ResourceExportBase() : handle(R::ResourceNew((R *)this)) {}
  // because this function is called by the host via Dtor we must not deregister
  ~ResourceExportBase() {}
  ResourceExportBase(ResourceExportBase const &) = delete;
  ResourceExportBase(ResourceExportBase &&) = delete;
  ResourceExportBase &operator=(ResourceExportBase &&b) = delete;
  ResourceExportBase &operator=(ResourceExportBase const &) = delete;
  handle_t get_handle() const { return handle; }
  handle_t into_handle() {
    handle_t result = handle;
    handle = invalid;
    return result;
  }
};

/// @brief A Resource imported from the host (guest side)
///
/// Wraps the identifier and can be forwarded but not duplicated
class ResourceImportBase {
public:
#ifdef WIT_SYMMETRIC
  typedef uint8_t *handle_t;
  static constexpr handle_t invalid = nullptr;
#else
  typedef int32_t handle_t;
  static const handle_t invalid = -1;
#endif

protected:
  handle_t handle;

public:
  ResourceImportBase(handle_t h = invalid) : handle(h) {}
  ResourceImportBase(ResourceImportBase &&r) : handle(r.handle) {
    r.handle = invalid;
  }
  ResourceImportBase(ResourceImportBase const &) = delete;
  void set_handle(handle_t h) { handle = h; }
  handle_t get_handle() const { return handle; }
  handle_t into_handle() {
    handle_t h = handle;
    handle = invalid;
    return h;
  }
  ResourceImportBase &operator=(ResourceImportBase &&r) {
    assert(handle == invalid);
    handle = r.handle;
    r.handle = invalid;
    return *this;
  }
  ResourceImportBase &operator=(ResourceImportBase const &r) = delete;
};
} // namespace wit
