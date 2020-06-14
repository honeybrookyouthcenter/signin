using Microsoft.AppCenter;
using Microsoft.AppCenter.Crashes;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Windows.ApplicationModel.Core;
using Windows.Storage;
using Windows.UI.Core;
using Windows.UI.Popups;
using YouthCenterSignIn.Logic.Data;
using MsG = Microsoft.Graph;

namespace YouthCenterSignIn
{
    public class UwpDataProvider : DataProvider
    {
        public override async Task ShowMessage(string message = null, Exception exception = null)
        {
            if (exception != null)
                Crashes.TrackError(exception, new Dictionary<string, string> { { "Message", message } });

            if (!string.IsNullOrWhiteSpace(message))
            {
                await CoreApplication.MainView.CoreWindow.Dispatcher.RunAsync(CoreDispatcherPriority.Normal, async () =>
                {
                    await new MessageDialog(message).ShowAsync();
                });
            }
        }

        #region People

        internal Graph Graph { get; } = new Graph();

        internal async Task ShowAuthenticationMessage()
        {
            await ShowMessage("Please login to a Microsoft account on the logs page.");
        }

        public override async Task<List<Person>> GetPeople()
        {
            var contacts = await Graph.GetContacts();

            return contacts
                .Select(c => ContactToPerson(c))
                .Where(c => c != null)
                .ToList();
        }

        Person ContactToPerson(MsG.Contact contact)
        {
            try
            {
                string id = contact.Id;
                string firstName = contact.GivenName;
                string lastName = contact.Surname;
                string notes = contact.PersonalNotes;

                return new Person(id, firstName, lastName, notes);
            }
            catch (Exception ex)
            {
                _ = ShowMessage(exception: ex);
                return null;
            }
        }

        public override async Task<string> SavePerson(Person person)
        {
            var contact = PersonToContact(person);
            if (string.IsNullOrWhiteSpace(contact.Id))
                return (await Graph.SaveContact(contact)).Id;
            else
                return (await Graph.UpdateContact(contact)).Id;
        }

        MsG.Contact PersonToContact(Person person)
        {
            var contact = new MsG.Contact();
            if (string.IsNullOrWhiteSpace(person.Id))
            {
                contact.GivenName = person.FirstName;
                contact.Surname = person.LastName;
                contact.Birthday = person.BirthDate;
            }
            else
            {
                contact.Id = person.Id;
            }
            contact.PersonalNotes = person.Notes;
            contact.HomePhones = new List<string>
            {
                person.Guardian?.PhoneNumber
            };
            contact.HomeAddress = person.Address.ToContactAddress();

            return contact;
        }

        #endregion

        #region Json

        protected override async Task<string> GetJsonFileContent(string fileName)
        {
            var rootFolder = await GetRootFolder();
            if (rootFolder == null)
                return null;

            var item = await rootFolder.TryGetItemAsync(fileName);
            if (item is IStorageFile file)
                return await FileIO.ReadTextAsync(file);
            else
                return null;
        }

        public async Task<StorageFolder> GetRootFolder()
        {
            try
            {
                var userPath = UserDataPaths.GetForUser(App.User).Profile;
                var userFolder = await StorageFolder.GetFolderFromPathAsync(userPath);
                var oneDriveFolder = await userFolder.GetFolderAsync("OneDrive");
                return await oneDriveFolder.CreateFolderAsync("Youth Center Sign In", CreationCollisionOption.OpenIfExists);
            }
            catch (UnauthorizedAccessException)
            {
                await ShowMessage("Could not access the folder, please give permission to the app.");
                await Windows.System.Launcher.LaunchUriAsync(new Uri("ms-settings:privacy-broadfilesystemaccess"));
                return null;
            }
        }

        protected override string GetJsonSetting(string key)
        {
            return (string)ApplicationData.Current.LocalSettings.Values[key];
        }

        protected override async Task SetJsonFileContent(string fileName, string json)
        {
            var rootFolder = await GetRootFolder();
            if (rootFolder == null)
                return;

            var file = await rootFolder.CreateFileAsync(fileName, CreationCollisionOption.OpenIfExists);
            await FileIO.WriteTextAsync(file, json);
        }

        protected override void SetJsonSetting(string key, string json)
        {
            ApplicationData.Current.LocalSettings.Values[key] = json;
        }

        #endregion
    }

    static class ContactExtensions
    {
        #region Addresses

        public static Address ToFullAddress(this MsG.PhysicalAddress address)
        {
            if (address == null)
                return null;

            return new Address(address.Street, city: address.City, state: address.State);
        }

        public static MsG.PhysicalAddress ToContactAddress(this Address address)
        {
            return new MsG.PhysicalAddress()
            {
                Street = address.StreetAddress,
                City = address.City,
                State = address.State
            };
        }

        #endregion
    }
}
