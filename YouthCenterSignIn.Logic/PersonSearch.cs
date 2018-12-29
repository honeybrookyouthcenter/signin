using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Linq;
using System.Threading.Tasks;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn.Logic
{
    public class PersonSearch : ReadOnlyObservableCollection<Person>, INotifyPropertyChanged
    {
        public PersonSearch() : base(new ObservableCollection<Person>())
        { }

        public Task SearchTask { get; private set; }
        string searchText;
        public string SearchText
        {
            get => searchText;
            set
            {
                searchText = value;

                SearchTask = UpdateSearchResults();

                OnPropertyChanged();
            }
        }

        private async Task UpdateSearchResults()
        {
            var search = SearchText?.ToLower();
            if (string.IsNullOrWhiteSpace(search))
            {
                Items.Clear();
                return;
            }

            var results = (await Person.GetPeople())
                .Where(p => p.FullName.ToLower().Contains(search))
                .OrderByDescending(p => p.FullName.ToLower().StartsWith(search))
                .Take(8);

            foreach (var person in Items.ToArray())
            {
                if (!results.Contains(person))
                    Items.Remove(person);
            }

            foreach (var person in results)
            {
                if (!Items.Contains(person))
                    Items.Add(person);
            }
        }


        public new event PropertyChangedEventHandler PropertyChanged;
        protected void OnPropertyChanged([System.Runtime.CompilerServices.CallerMemberName] string propertyName = "")
        {
            PropertyChangedEventArgs args = new PropertyChangedEventArgs(propertyName);
            base.OnPropertyChanged(args);
            PropertyChanged?.Invoke(this, args);
        }
    }
}